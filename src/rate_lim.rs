use crate::errors::{CustomError, CustomResult};
use actix_web::dev::{MessageBody, Service, ServiceRequest, ServiceResponse, Transform};
use chrono::{Timelike, Utc};
use core::future::Future;
use redis::aio::ConnectionManager;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct RateLimiter {
    connection_manager: ConnectionManager,
}

impl RateLimiter {
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self { connection_manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    type Response = S::Response;
    type Error = S::Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let mw = RateLimiterMiddleware::new(self.connection_manager.clone(), service);
        futures::future::ready(Ok(mw))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    connection_manager: ConnectionManager,
}

impl<S> RateLimiterMiddleware<S> {
    pub fn new(connection_manager: ConnectionManager, service: S) -> Self {
        Self {
            service,
            connection_manager,
        }
    }
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RateLimiterFuture<S, B>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let address = req.peer_addr();
        let fut = self.service.call(req);
        let connections_result = futures::executor::block_on(get_connections_count(
            address,
            self.connection_manager.clone(),
        ));
        RateLimiterFuture::new(fut, connections_result)
    }
}

#[pin_project::pin_project]
pub struct RateLimiterFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    #[pin]
    fut: S::Future,
    _phantom: PhantomData<B>,
    connections_result: CustomResult<u64>,
}

impl<S, B> RateLimiterFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    const CONNECTIONS_LIMIT: u64 = 5;

    pub fn new(fut: S::Future, connections_result: CustomResult<u64>) -> Self {
        Self {
            fut,
            _phantom: Default::default(),
            connections_result,
        }
    }
}

impl<S, B> Future for RateLimiterFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    type Output = Result<ServiceResponse<B>, actix_web::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let connections = match &self.connections_result {
            Ok(c) => *c,
            Err(e) => return Poll::Ready(Err(e.clone().into())),
        };

        if connections > Self::CONNECTIONS_LIMIT {
            return Poll::Ready(Err(CustomError::TooManyRequests {
                actual: connections,
                max: Self::CONNECTIONS_LIMIT,
            }
            .into()));
        }

        let this = self.project();
        let res = match futures::ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };
        Poll::Ready(Ok(res))
    }
}

const KEY_PREFIX: &str = "RATE_LIMIT_";

async fn get_connections_count(
    address: Option<SocketAddr>,
    mut cm: ConnectionManager,
) -> CustomResult<u64> {
    let addr =
        address.ok_or_else(|| CustomError::InternalError("Can't parse peer address".into()))?;

    let current_minute = Utc::now().minute();
    let key = format!("{}:{}:{}", KEY_PREFIX, addr, current_minute);

    let mut pipeline = redis::pipe();

    let (connections_count, _) = pipeline
        .atomic()
        .incr(&key, 1)
        .expire(&key, 60)
        .query_async::<_, (u64, u64)>(&mut cm)
        .await?;

    Ok(connections_count)
}
