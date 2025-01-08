use std::{future::Future, pin::Pin};

use axum::extract::Request;
use axum::response::Response;
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use super::{REQUEST_ID_HEADER, SERVER_TIME_HEADER};

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ServerTimeService { inner: service }
    }
}

#[derive(Clone)]
pub struct ServerTimeService<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut res: Response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());
            match elapsed.parse() {
                Ok(v) => {
                    res.headers_mut().insert(SERVER_TIME_HEADER, v);
                }
                Err(e) => {
                    warn!(
                        "Parse elapsed time failed: {} for request {:?}",
                        e,
                        res.headers().get(REQUEST_ID_HEADER)
                    );
                }
            }
            Ok(res)
        })
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
}
