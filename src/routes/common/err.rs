use aide::OperationIo;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongo::MongoError;
use thiserror::Error;

#[derive(OperationIo)]
#[aide(output)]
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("i/o error: {0}")]
    IO(#[from] std::io::Error),
    #[error("PostgreSQL error: {0}")]
    SQL(#[from] sea_orm::DbErr),
    #[error("MongoDB error: {0}")]
    Mongo(#[from] MongoError),
    #[error("session error: {0}")]
    Session(#[from] std::convert::Infallible),
    #[error("serde_json error: {0}")]
    Json(#[from] axum_sessions::async_session::serde_json::Error),
    #[error("axum multipart error: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("tokio task error: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("{0}")]
    Common(String),
    #[error("{0}")]
    BadReqest(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Mongo(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Forbidden(e) => (StatusCode::FORBIDDEN, e),
            Error::NotFound(e) => (StatusCode::NOT_FOUND, e),
            Error::Session(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Json(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Conflict(e) => (StatusCode::CONFLICT, e),
            Error::BadReqest(e) => (StatusCode::BAD_REQUEST, e),
            Error::Multipart(e) => (e.status(), e.body_text()),
            Error::IO(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Task(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::Common(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Error::SQL(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
        .into_response()
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
