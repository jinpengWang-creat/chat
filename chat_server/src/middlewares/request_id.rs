use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let request_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(id) => Some(id.clone()),
        None => {
            let id = Uuid::now_v7().to_string();
            match HeaderValue::from_str(&id) {
                Ok(id) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, id.clone());
                    Some(id)
                }
                Err(e) => {
                    warn!("Failed to set request id header: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;

    if let Some(id) = request_id {
        res.headers_mut().insert(REQUEST_ID_HEADER, id);
    };
    res
}
