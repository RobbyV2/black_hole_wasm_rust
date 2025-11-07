use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloResponse {
    message: String,
}

async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello World from Rust!".to_string(),
    })
}

pub fn routes() -> Router {
    Router::new().route("/hello", get(hello))
}
