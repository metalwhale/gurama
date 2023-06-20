use anyhow::{anyhow, Error, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router, Server,
};
use llm::{
    models::Llama, samplers::TopPTopK, InferenceFeedback, InferenceResponse, Model,
    VocabularySource,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{convert::Infallible, env, path::Path, sync::Arc};

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

#[tokio::main]
async fn main() -> Result<()> {
    let model_path = env::var("GURAMA_MODEL_PATH")?;
    let app_port = env::var("GURAMA_APP_PORT")?.parse::<i32>()?;
    let llama = llm::load::<Llama>(
        Path::new(&model_path),
        VocabularySource::Model,
        Default::default(),
        llm::load_progress_callback_stdout,
    )?;
    let state = Arc::new(AppState { llama });
    let app = Router::new()
        .route("/correct", post(correct))
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
    let mut session = state.llama.start_session(Default::default());
    let sentence = format!("'{}' can be corrected as:", correction.sentence);
    let mut corrected_sentence = String::new();
    match session.infer::<Infallible>(
        &state.llama,
        &mut rand::thread_rng(),
        &llm::InferenceRequest {
            prompt: (&sentence).into(),
            parameters: &llm::InferenceParameters {
                n_threads: num_cpus::get(),
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
        Ok(_) => Ok(Json(
            json!({ "corrected_sentence": corrected_sentence.trim() }),
        )),
        Err(e) => Err(AppError(anyhow!("{e}"))),
    }
}
