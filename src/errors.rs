use core::fmt;
use rocket::{
    http::Status,
    response::{status, Responder},
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub message: String,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiError {
    BadRequest,
    NotFound,
    Unauthorized,
    InternalServerError(String),
}

pub type Response<T> = Result<T, ApiError>;

impl From<stripe::StripeError> for ApiError {
    fn from(value: stripe::StripeError) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl From<surrealdb::Error> for ApiError {
    fn from(value: surrealdb::Error) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl From<base64::DecodeError> for ApiError {
    fn from(value: base64::DecodeError) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        ApiError::InternalServerError(value.to_string())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::BadRequest => write!(f, "Bad Request"),
            ApiError::NotFound => write!(f, "Not Found"),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::InternalServerError(ref message) => {
                write!(f, "Internal Server Error: {:?}", message)
            }
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = match self {
            ApiError::BadRequest => Status::BadRequest,
            ApiError::NotFound => Status::NotFound,
            ApiError::Unauthorized => Status::Unauthorized,
            _ => Status::InternalServerError,
        };

        status::Custom(
            status,
            Json(json!({"status": status, "error": self.to_string()})),
        )
        .respond_to(request)
    }
}
