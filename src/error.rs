use std::io;

use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error fetching image")]
    Download,
    #[error("IO error")]
    IO(#[from] io::Error),
    #[error("Conversion error")]
    Conversion(#[from] image::ImageError),
    #[error("Missing dimensions to resize")]
    ResizeEmptyDimension,
    #[error("Invalid file format")]
    InvalidImageFormat,
    #[error("Host not allowed")]
    HostNotAllowed,
    #[error("Unknown error")]
    Unknown,
}

//NOTE: maybe this should be moved to http as it doesn't concern the error type?
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorMessage {
    error: String,
}
