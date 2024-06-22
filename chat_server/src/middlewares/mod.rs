mod chat;

use axum::{middleware::from_fn, Router};
pub use chat::verify_is_chat_member;
use chat_core::{set_request_id, ServerTimeLayer};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

pub fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    )
                    .on_failure(
                        DefaultOnFailure::default()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
