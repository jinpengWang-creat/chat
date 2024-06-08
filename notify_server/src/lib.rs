use axum::{routing::get, Router};
use sse::sse_handler;

mod sse;

pub fn get_router() -> Router {
    Router::new().route("/events", get(sse_handler))
}
