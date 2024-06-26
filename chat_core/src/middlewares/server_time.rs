use axum::{extract::Request, http::HeaderValue, response::Response};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use crate::middlewares::REQUEST_ID_HEADER;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed = format!("{:?}us", start.elapsed().as_micros());
            if let Ok(server_time) = HeaderValue::from_str(&elapsed) {
                response
                    .headers_mut()
                    .insert(SERVER_TIME_HEADER, server_time);
            } else {
                warn!(
                    "Parse elapsed time failed: {} for request: {:?}",
                    elapsed,
                    response.headers().get(REQUEST_ID_HEADER)
                );
            }
            Ok(response)
        })
    }
}
