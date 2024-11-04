use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::handler::{health_checker_handler, prove_and_push_handler, prove_handler};

pub fn create_router(api_key: Arc<String>) -> Router {
    let protected_routes = Router::new()
        .route("/api/prove-and-push", post(prove_and_push_handler))
        .route("/api/prove", post(prove_handler))
        .route_layer(middleware::from_fn_with_state(
            api_key.clone(),
            api_key_middleware,
        ));

    Router::new()
        .route("/api/healthz", get(health_checker_handler))
        .merge(protected_routes)
}

pub async fn api_key_middleware(
    State(expected_api_key): State<Arc<String>>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Specify the correct return type
    // Extract the API key from the headers
    if let Some(api_key) = req.headers().get("x-api-key") {
        if api_key == expected_api_key.as_str() {
            // API keys match, proceed to the next handler
            return next.run(req).await;
        }
    }

    (StatusCode::UNAUTHORIZED, "Invalid or missing API key").into_response()
}
