use actix_web::{
    error,
    http::{header::ContentType, header::ToStrError, StatusCode},
    HttpResponse,
};
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error::{DatabaseError, NotFound};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    RecordAlreadyExists,
    RecordNotFound,
    DatabaseError(diesel::result::Error),
    BlockingError(String),
    R2d2Error(r2d2::Error),
    UuidParseError(uuid::Error),
    AuthError,
    HeaderParse(String),
    JWKSFetchError,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    err: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::RecordAlreadyExists => write!(f, "This record violates a unique constraint"),
            AppError::RecordNotFound => write!(f, "This record does not exist"),
            AppError::DatabaseError(e) => write!(f, "Database error: {:?}", e),
            AppError::BlockingError(e) => write!(f, "The running operation was blocked: {:?}", e),
            AppError::R2d2Error(e) => write!(f, "Database connection pool error: {:?}", e),
            AppError::UuidParseError(e) => write!(f, "UUID parse error: {:?}", e),
            AppError::AuthError =>  write!(f, "Unauthorized request. Pass user access token in request header."),
            AppError::HeaderParse(e) => write!(f, "Header parse error: {:?}", e),
            AppError::JWKSFetchError => write!(f, "Could not fetch JWKS"),
        }
    }
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::RecordAlreadyExists => StatusCode::BAD_REQUEST,
            AppError::RecordNotFound => StatusCode::NOT_FOUND,
            AppError::DatabaseError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BlockingError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::R2d2Error(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UuidParseError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError => StatusCode::UNAUTHORIZED,
            AppError::HeaderParse(_e) => StatusCode::BAD_REQUEST,
            AppError::JWKSFetchError => StatusCode::BAD_REQUEST
        }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            DatabaseError(UniqueViolation, _) => AppError::RecordAlreadyExists,
            NotFound => AppError::RecordNotFound,
            _ => AppError::DatabaseError(e),
        }
    }
}

impl From<error::BlockingError> for AppError {
    fn from(e: error::BlockingError) -> Self {
        AppError::BlockingError(e.to_string())
    }
}

impl From<r2d2::Error> for AppError {
    fn from(e: r2d2::Error) -> Self {
        AppError::R2d2Error(e)
    }
}

impl From<uuid::Error> for AppError {
    fn from(e: uuid::Error) -> Self {
        AppError::UuidParseError(e)
    }
}

impl From<ToStrError> for AppError {
    fn from(e: ToStrError) -> Self {
        AppError::HeaderParse(e.to_string())
    }
}