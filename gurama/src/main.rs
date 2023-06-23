use std::{convert::Infallible, env, path::Path, sync::Arc, thread};

use anyhow::{anyhow, Error, Result};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Form, Json, Router, Server,
};
use llm::{
    models::Llama, samplers::TopPTopK, InferenceFeedback, InferenceResponse, Model,
    VocabularySource,
};
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};

struct AppError(Error);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

struct AppState {
    llama: Llama,
}

#[derive(Deserialize)]
struct Correction {
    sentence: String,
}

#[derive(Deserialize)]
struct SlackCorrection {
    // See: https://api.slack.com/interactivity/slash-commands#app_command_handling
    text: String,
    response_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Number of cpus: {}", num_cpus::get());
    let model_path = env::var("GURAMA_MODEL_PATH")?;
    let app_port = env::var("GURAMA_APP_PORT")?.parse::<i32>()?;
    let app_prefix = env::var("GURAMA_APP_PREFIX").unwrap_or("".to_string());
    let llama = llm::load::<Llama>(
        Path::new(&model_path),
        VocabularySource::Model,
        Default::default(),
        llm::load_progress_callback_stdout,
    )?;
    let state = Arc::new(AppState { llama });
    let app = Router::new()
        .route(&format!("{app_prefix}/correct"), post(correct))
        .route(&format!("{app_prefix}/correct_slack"), post(correct_slack))
        .with_state(state);
    Server::bind(&format!("0.0.0.0:{app_port}").parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn correct(
    State(state): State<Arc<AppState>>,
    Json(correction): Json<Correction>,
) -> Result<Json<Value>, AppError> {
    match infer(&state, &correction.sentence) {
        Ok(s) => Ok(Json(json!({ "corrected_sentence": s }))),
        Err(e) => Err(AppError(e)),
    }
}

async fn correct_slack(
    State(state): State<Arc<AppState>>,
    Form(slack_correction): Form<SlackCorrection>,
) -> Result<Json<Value>, AppError> {
    let message = format!("Correcting: \"{}\"...", slack_correction.text);
    thread::spawn(move || correct_slack_infer(state, slack_correction));
    Ok(Json(json!({
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message,
                }
            }
        ]
    })))
}

fn correct_slack_infer(state: Arc<AppState>, slack_correction: SlackCorrection) {
    let client = Client::new();
    let response_text = match infer(&state, &slack_correction.text) {
        Ok(s) => format!("Corrected: {}", s),
        Err(e) => e.to_string(),
    };
    match client
        .post(&slack_correction.response_url)
        .json::<Value>(&json!({
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": response_text,
                    }
                }
            ]
        }))
        .send()
    {
        Ok(_) => {}
        Err(e) => println!("{e}"),
    };
}

fn infer(state: &AppState, sentence: &str) -> Result<String> {
    let mut session = state.llama.start_session(Default::default());
    let sentence = format!("\"{}\" can be corrected as:", sentence);
    let n_threads = env::var("GURAMA_APP_THREADS")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap_or(num_cpus::get());
    let mut corrected_sentence = String::new();
    match session.infer::<Infallible>(
        &state.llama,
        &mut rand::thread_rng(),
        &llm::InferenceRequest {
            prompt: (&sentence).into(),
            parameters: &llm::InferenceParameters {
                n_threads,
                sampler: Arc::new(TopPTopK {
                    temperature: 0.01,
                    ..Default::default()
                }),
                ..Default::default()
            },
            play_back_previous_tokens: false,
            maximum_token_count: None,
        },
        &mut Default::default(),
        |r| match r {
            InferenceResponse::InferredToken(t) => {
                corrected_sentence.push_str(&t);
                Ok(InferenceFeedback::Continue)
            }
            _ => Ok(InferenceFeedback::Continue),
        },
    ) {
        Ok(_) => Ok(corrected_sentence.trim().to_string()),
        Err(e) => Err(anyhow!("{e}")),
    }
}
