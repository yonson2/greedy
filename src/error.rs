use std::io;

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
    #[error("Unknown error")]
    Unknown,
}
