use actix_web::body::Body;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use redis::RedisError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("MongoDB error: {0}")]
    MongoDbError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Endpoint is not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Too many requests: {actual} requests when {max} allowed")]
    TooManyRequests { actual: u64, max: u64 },
}

pub type CustomResult<T> = Result<T, CustomError>;

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MongoDbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse<Body> {
        log::error!("Error: {}", self.to_string());

        let error_response = ErrorResponse(self.to_string());

        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .json(error_response)
    }
}

#[derive(Serialize)]
struct ErrorResponse(String);

impl From<mongodb::error::Error> for CustomError {
    fn from(source: mongodb::error::Error) -> Self {
        Self::MongoDbError(source.to_string())
    }
}

impl From<mongodb::bson::de::Error> for CustomError {
    fn from(source: mongodb::bson::de::Error) -> Self {
        Self::MongoDbError(source.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for CustomError {
    fn from(source: mongodb::bson::ser::Error) -> Self {
        Self::MongoDbError(source.to_string())
    }
}

impl From<mongodb::bson::oid::Error> for CustomError {
    fn from(source: mongodb::bson::oid::Error) -> Self {
        Self::NotFound(source.to_string())
    }
}

impl From<RedisError> for CustomError {
    fn from(source: RedisError) -> Self {
        Self::RedisError(source.to_string())
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(source: serde_json::Error) -> Self {
        Self::InternalError(source.to_string())
    }
}
