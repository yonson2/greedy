use std::io::Cursor;

use image::{load_from_memory, ImageFormat};

use crate::error::Error;

#[derive(Clone, Copy)]
pub enum Format {
    Avif,
    Png,
    Webp,
}

impl From<Format> for ImageFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::Avif => ImageFormat::Avif,
            Format::Png => ImageFormat::Png,
            Format::Webp => ImageFormat::WebP,
        }
    }
}

/// `download_image` is a little helper function that takes a url and returns
/// a Vec<u8> with its contents.
fn download_image(url: &str) -> Result<Vec<u8>, Error> {
    // Send a GET request to download the image
    let response = ureq::get(url).call().map_err(|e| {
        tracing::error!(error = e.to_string(), "Error downloading image");
        Error::Download
    })?;

    // Read the response body into a Vec<u8>
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn convert_file(file: &[u8], to: Format) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(file)?;
    let mut converted_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut converted_img, to.into())?;
    Ok(converted_img.get_ref().clone())
}

#[cfg(test)]
mod test {
    use super::*;

    impl Format {
        fn iter() -> impl Iterator<Item = Format> {
            [Format::Avif, Format::Png, Format::Webp].iter().copied()
        }
    }

    #[test]
    fn download() -> Result<(), Error> {
        let url = "https://placehold.co/1x1";
        let file = download_image(url)?;

        assert!(!file.is_empty());
        Ok(())
    }

    // let avif_file = download_image("https://raw.githubusercontent.com/link-u/avif-sample-images/refs/heads/master/kimono.avif")?;
    #[test]
    fn convert_png() -> Result<(), Error> {
        let file = download_image("https://placehold.co/1x1.png")?;

        for format in [Format::Avif, Format::Webp].iter() {
            let new_file = convert_file(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    #[test]
    fn convert_avif() -> Result<(), Error> {
        let file = download_image("https://raw.githubusercontent.com/link-u/avif-sample-images/refs/heads/master/kimono.avif")?;

        for format in [Format::Png, Format::Webp].iter() {
            let new_file = convert_file(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    #[test]
    fn convert_webp() -> Result<(), Error> {
        let file = download_image("https://placehold.co/1x1.webp")?;

        for format in [Format::Png, Format::Avif].iter() {
            let new_file = convert_file(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
}
