use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use image::{guess_format, ImageFormat};
use serde::Deserialize;

use crate::{
    error::Error,
    http::Result,
    image::{download, transform, Dimension, Format, Height, Operation, Saved, Width},
};

use super::{ApiMessage, ApiState};

pub async fn process_and_serve(
    Path(url): Path<String>,
    Query(opts): Query<ImageOptions>,
    State(state): State<ApiState>,
) -> Result<impl IntoResponse> {
    //TODO: turn this into a middleware.
    allowed_host(&url, &state.whitelist)?;

    let cache = state.cache;
    // construct our savedimage from the request.
    let img = get_saved_image(&url, opts.width, opts.height, opts.format);

    // check cache or save to cache.
    let key = img.to_string();
    let data = if let Some(i) = cache.get(&key).await {
        i
    } else {
        let operations = get_operations(&opts);
        let data = transform(&download(&url)?, &operations)?;
        cache.insert(key, data.clone()).await;
        data
    };

    let content_type = if let Some(f) = img.format {
        f.content_type().to_string()
    } else {
        guess_content_type(&data)?
    };
    let headers = [
        (header::CACHE_CONTROL, "max-age=31536000".to_string()),
        (header::CONTENT_TYPE, content_type),
    ];

    Ok((StatusCode::OK, headers, data))
}

pub async fn preload(
    Path(url): Path<String>,
    Query(opts): Query<ImageOptions>,
    State(state): State<ApiState>,
) -> Result<impl IntoResponse> {
    allowed_host(&url, &state.whitelist)?;
    let cache = state.cache;
    let img = get_saved_image(&url, opts.width, opts.height, opts.format);
    let key = img.to_string();
    if cache.get(&key).await.is_none() {
        let operations = get_operations(&opts);
        let data = transform(&download(&url)?, &operations)?;
        cache.insert(key, data.clone()).await;
    }

    Ok((
        StatusCode::OK,
        Json(ApiMessage {
            message: "OK".to_string(),
        }),
    ))
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

fn allowed_host(url: &str, whitelist: &[String]) -> Result<(), Error> {
    let parsed = url::Url::parse(url).map_err(|e| {
        tracing::error!(
            error = e.to_string(),
            "Error parsing url to check whitelist"
        );
        Error::HostNotAllowed
    })?;

    let host = parsed.host_str().unwrap_or("");

    if whitelist.iter().any(|h| h == host) {
        Ok(())
    } else {
        Err(Error::HostNotAllowed)
    }
}

fn get_saved_image(
    url: &str,
    width: Option<Width>,
    height: Option<Height>,
    format: Option<Format>,
) -> Saved {
    Saved {
        url: url.to_owned(),
        dimensions: Dimension(width, height),
        format,
    }
}
