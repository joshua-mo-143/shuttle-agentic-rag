use std::env::set_var;
use axum::{body::Body, middleware::{self, Next}, Json};
use serde_json::json;
use axum::{
    extract::State,
    http::{StatusCode, Request},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use agents::MyAgent;

use qdrant_client::client::QdrantClient;
use serde::Deserialize;

pub mod agents;
pub mod files;

use files::File;
use shuttle_runtime::SecretStore;



// Middleware function to log requests
async fn log_requests(req: Request<Body>, next: Next) -> impl IntoResponse {
    println!("Handling {} {}", req.method(), req.uri());
    let res = next.run(req).await;
    println!("Response Status: {}", res.status());
    res
}


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
        Ok(res) => (StatusCode::OK, Json(json!({"response": res}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Something went wrong: {e}")}))
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
    
    let file = match File::new("test.csv".into()) {
        Ok(file) => file.parse(),
        Err(e) => {
            eprintln!("Failed to read file: {e}");
            return Err(e.into());
        },
    };

    let state = AppState {
        agent: MyAgent::new(qdrant_client),
    };

    state.agent.embed_document(file).await?;

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/prompt", post(prompt))
        .layer(middleware::from_fn(log_requests))  // Adding the middleware here
        .with_state(state);

    Ok(router.into())
}

