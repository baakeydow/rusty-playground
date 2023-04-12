//! Error type for DTK
use actix_web::http::header::ToStrError;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
/// Error type for DTK
pub struct DtkError(String);

impl std::convert::From<&str> for DtkError {
    fn from(error: &str) -> Self {
        DtkError(error.to_string())
    }
}

impl std::convert::From<ToStrError> for DtkError {
    fn from(error: ToStrError) -> Self {
        DtkError(error.to_string())
    }
}

impl std::convert::From<std::io::Error> for DtkError {
    fn from(error: std::io::Error) -> Self {
        DtkError(error.to_string())
    }
}

impl ResponseError for DtkError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Forbidden().json("Invalid token")
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::FORBIDDEN
    }
}

impl std::fmt::Display for DtkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}