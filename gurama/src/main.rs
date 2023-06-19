use llm::Model;
use std::env;
use std::io::Write;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let prompt = &args[1];
    let llama = llm::load::<llm::models::Llama>(
        std::path::Path::new("/usr/src/app/model/openbuddy-openllama-7b-v5-q4_0.bin"),
        llm::VocabularySource::Model,
        Default::default(),
        llm::load_progress_callback_stdout,
    )
    .unwrap_or_else(|err| panic!("Failed to load model: {err}"));
    let mut session = llama.start_session(Default::default());
    let res = session.infer::<std::convert::Infallible>(
        &llama,
        &mut rand::thread_rng(),
        &llm::InferenceRequest {
            prompt: prompt.into(),
            parameters: &llm::InferenceParameters {
                n_threads: num_cpus::get(),
                sampler: Arc::new(llm::samplers::TopPTopK {
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
            llm::InferenceResponse::PromptToken(t) | llm::InferenceResponse::InferredToken(t) => {
                print!("{t}");
                std::io::stdout().flush().unwrap();
                Ok(llm::InferenceFeedback::Continue)
            }
            _ => Ok(llm::InferenceFeedback::Continue),
        },
    );
    match res {
        Ok(result) => println!("\n\nInference stats:\n{result}"),
        Err(err) => println!("\n{err}"),
    }
}
