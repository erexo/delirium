use async_trait::async_trait;
use poem::{Endpoint, Error, IntoResponse, Middleware, Request, Response};
use tracing::error;

#[derive(Default)]
pub struct TraceError;

impl<E: Endpoint> Middleware<E> for TraceError {
    type Output = TraceErrorEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        TraceErrorEndpoint { inner: ep }
    }
}

pub struct TraceErrorEndpoint<E> {
    inner: E,
}

#[async_trait]
impl<E: Endpoint> Endpoint for TraceErrorEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        match self.inner.call(req).await {
            Ok(res) => Ok(res.into_response()),
            Err(err) => {
                if err.status().is_client_error() {
                    Err(err)
                } else {
                    error!("{}", err);
                    Err(Error::from_status(err.status()))
                }
            }
        }
    }
}
