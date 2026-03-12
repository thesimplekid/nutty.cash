use crate::server::nut24::handle_paycode_api;
use crate::server::state::AppState;
use axum::{
    http::{header, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/paycode", post(handle_paycode_api))
        .route("/SKILL.md", get(handle_skill_md))
        .route("/v1/SKILL.md", get(handle_skill_md))
        .route("/api/v1/SKILL.md", get(handle_skill_md))
        .with_state(state)
}

async fn handle_skill_md() -> impl IntoResponse {
    let content = include_str!("../../SKILL.md");
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/markdown"),
        )],
        content,
    )
}
