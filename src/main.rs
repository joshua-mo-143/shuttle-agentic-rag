use std::env::set_var;

use agents::MyAgent;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use qdrant_client::client::QdrantClient;
use serde::Deserialize;

pub mod agents;
pub mod files;

use files::File;
use shuttle_runtime::SecretStore;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
pub struct Prompt {
    prompt: String,
}

#[derive(Clone)]
pub struct AppState {
    agent: MyAgent,
}

async fn prompt(State(state): State<AppState>, Json(json): Json<Prompt>) -> impl IntoResponse {
    match state.agent.prompt(&json.prompt).await {
        Ok(res) => (StatusCode::OK, res),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {e}"),
        ),
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_qdrant::Qdrant(
        cloud_url = "{secrets.QDRANT_URL}",
        api_key = "{secrets.QDRANT_API_TOKEN}"
    )]
    qdrant_client: QdrantClient,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    secrets.into_iter().for_each(|x| {
        set_var(x.0, x.1);
    });

    let file = File::new("test.csv".into())?;

    let state = AppState {
        agent: MyAgent::new(qdrant_client),
    };

    state.agent.embed_document(file).await?;

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/prompt", post(prompt))
        .with_state(state);

    Ok(router.into())
}
