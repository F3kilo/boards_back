use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Fail to complete request: ")]
pub enum CustomError {
    #[error("MongoDB error: {0}")]
    MongoDbError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Endpoint is not found: {0}")]
    NotFound(String),
    #[error("Too many requests: {actual} requests when {max} allowed")]
    TooManyRequests { actual: u64, max: u64 },
}

pub type CustomResult<T> = Result<T, CustomError>;

impl ResponseError for CustomError {}