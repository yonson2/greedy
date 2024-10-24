use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use image::{guess_format, ImageFormat};
use serde::Deserialize;

use crate::{
    cache::Cache,
    error::Error,
    http::Result,
    image::{download, transform, Dimension, Format, Height, Operation, Saved, Width},
};

#[debug_handler]
pub async fn process_and_serve(
    Path(url): Path<String>,
    Query(opts): Query<ImageOptions>,
    State(cache): State<Cache>,
) -> Result<Response> {
    // Steps:
    // 1: try to get from cache
    // 2: download image
    // 3: transform image (get operations from query)
    // 4: save to cache

    // construct our savedimage from the request.
    let img = Saved {
        url: url.to_string(),
        dimensions: Dimension(opts.width, opts.height),
        format: opts.format,
    };

    let data = if let Some(i) = cache.get(&img.to_string()).await {
        i
    } else {
        let operations = get_operations(&opts);
        let data = transform(&download(&url)?, &operations)?;
        cache.insert(img.to_string(), data.clone()).await;
        data
    };

    let content_type = guess_content_type(&data)?;
    let headers = [(header::CONTENT_TYPE, content_type)];
    let response = (StatusCode::OK, headers, data);
    Ok(response.into_response())
}

#[derive(Deserialize, Debug)]
pub struct ImageOptions {
    format: Option<Format>,
    width: Option<Width>,
    height: Option<Height>,
}

fn guess_content_type(image: &[u8]) -> Result<String, Error> {
    let format = guess_format(image)?;
    // only supported types are https://www.iana.org/assignments/media-types/media-types.xhtml#image
    match format {
        ImageFormat::Png => Ok("image/png".into()),
        ImageFormat::Jpeg => Ok("image/jpeg".into()),
        ImageFormat::Gif => Ok("image/gif".into()),
        ImageFormat::WebP => Ok("image/webp".into()),
        ImageFormat::Tiff => Ok("image/tiff".into()),
        ImageFormat::Bmp => Ok("image/bmp".into()),
        ImageFormat::Ico => Ok("image/x-icon".into()), // https://stackoverflow.com/a/28300054
        ImageFormat::Avif => Ok("image/avif".into()),
        _ => Err(Error::InvalidImageFormat),
    }
}

fn get_operations(opts: &ImageOptions) -> Vec<Operation> {
    let mut operations = Vec::with_capacity(2);
    if let Some(f) = opts.format {
        operations.push(Operation::Convert(f));
    }
    match (opts.width, opts.height) {
        (None, None) => (),
        _ => operations.push(Operation::Resize(Dimension(opts.width, opts.height))),
    }
    operations
}
