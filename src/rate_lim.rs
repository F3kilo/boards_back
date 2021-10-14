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
        log::trace!("new_transform(&self, service: S) called");
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
        log::trace!("call(&self, req: ServiceRequest) called");
        let addr = req.peer_addr();
        let fut = self.service.call(req);
        RateLimiterFuture::new(fut, addr, self.connection_manager.clone())
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
    addr: Option<SocketAddr>,
    cm: ConnectionManager,
}

impl<S, B> RateLimiterFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    const CONNECTIONS_LIMIT: u64 = 500;

    pub fn new(fut: S::Future, addr: Option<SocketAddr>, cm: ConnectionManager) -> Self {
        Self {
            fut,
            _phantom: Default::default(),
            addr,
            cm,
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
        log::trace!("poll Rate lim called");

        let connects =
            futures::executor::block_on(get_connections_count(self.addr, self.cm.clone()));
        let requests_count = match connects {
            Ok(reqs) => reqs,
            Err(e) => return Poll::Ready(Err(e.into())),
        };

        log::trace!("connections count future finished");

        if requests_count > Self::CONNECTIONS_LIMIT {
            log::info!("Too many requests. Connection dropped.");
            return Poll::Ready(Err(CustomError::TooManyRequests {
                actual: requests_count,
                max: Self::CONNECTIONS_LIMIT,
            }
            .into()));
        }

        log::trace!("Requests count check passed.");

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
    addr: Option<SocketAddr>,
    mut cm: ConnectionManager,
) -> CustomResult<u64> {
    let addr = addr.ok_or_else(|| CustomError::InternalError("Can't parse peer address".into()))?;

    let current_minute = Utc::now().minute();
    let key = format!("{}:{}:{}", KEY_PREFIX, addr, current_minute);

    let mut pipeline = redis::pipe();

    log::trace!("connections count request starting");

    let (connections_count, other) = pipeline
        .atomic()
        .incr(&key, 1)
        .expire(&key, 60)
        .query_async::<_, (u64, u64)>(&mut cm)
        .await?;

    log::trace!(
        "connections count request finished: {}, {}",
        connections_count,
        other
    );

    Ok(connections_count)
}
