use axum::{
    routing::{get, post},
    Router,
};

use crate::handler::{health_checker_handler, prove_and_push_handler, prove_handler};

pub fn create_router() -> Router {
    Router::new()
        .route("/api/healthz", get(health_checker_handler))
        .route("/api/prove-and-push", post(prove_and_push_handler))
        .route("/api/prove", post(prove_handler))
}
