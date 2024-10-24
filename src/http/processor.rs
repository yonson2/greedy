use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use image::{guess_format, ImageFormat};
use serde::Deserialize;

use crate::{
    cache::Cache,
    error::{self, Error},
    http::{ApiMessage, Result},
    image::{download_image, transform, Dimension, Format, Height, Operation, SavedImage, Width},
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
    let img = SavedImage {
        url: url.to_string(),
        dimensions: Dimension(opts.width, opts.height),
        format: opts.format,
    };

    let data = match cache.get(&img.to_string()).await {
        Some(i) => i,
        // If the cache is empty, process and save.
        None => {
            let ops = get_operations(&opts);
            let data = transform(&download_image(&url)?, &ops)?;
            cache.insert(img.to_string(), data.clone()).await;
            data
        }
    };

    let content_type = guess_content_type(&data)?;
    let headers = [(header::CONTENT_TYPE, content_type.clone())];
    let response = (StatusCode::OK, headers, data.clone());
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
    let mut ops = Vec::with_capacity(2);
    if let Some(f) = opts.format {
        ops.push(Operation::Convert(f));
    }
    match (opts.width, opts.height) {
        (None, None) => (),
        _ => ops.push(Operation::Resize(Dimension(opts.width, opts.height))),
    }
    ops
}
