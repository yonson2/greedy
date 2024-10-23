use std::{fmt::Display, io::Cursor};

use image::{guess_format, load_from_memory};

use crate::error::Error;

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

//TODO: tests?
/// `resize_image` scales _*down*_ the given image.
fn resize_image(file: &[u8], d: Dimension) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(file)?;
    let (width, height) = match d {
        Dimension(Some(Width(w)), Some(Height(h))) => (w, h),
        Dimension(Some(Width(w)), None) => (w, img.height()),
        Dimension(None, Some(Height(h))) => (img.width(), h),
        Dimension(None, None) => return Err(Error::ResizeEmptyDimension),
    };
    let img = img.thumbnail(width, height);
    let mut resized_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut resized_img, guess_format(file)?)?;
    Ok(resized_img.get_ref().clone())
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Avif,
    Png,
    Webp,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Avif => "avif",
            Self::Png => "png",
            Self::Webp => "webp",
        };
        write!(f, "{name}")
    }
}

impl From<Format> for image::ImageFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::Avif => image::ImageFormat::Avif,
            Format::Png => image::ImageFormat::Png,
            Format::Webp => image::ImageFormat::WebP,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Width(u32);
#[derive(Clone, Copy, Debug)]
struct Height(u32);
#[derive(Clone, Debug)]
struct Dimension(Option<Width>, Option<Height>);

//TODO: refacor into a macro?
impl Display for Width {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match (self.0, self.1) {
            (Some(w), Some(h)) => &format!("{w}_{h}"),
            (Some(w), None) => &format!("{w}_original"),
            (None, Some(h)) => &format!("original_{h}"),
            (None, None) => "original_original",
        };
        write!(f, "{output}")
    }
}

/// `SavedImage` holds the info about an image saved in cache.
#[derive(Clone, Debug)]
struct SavedImage {
    url: String,
    data: Vec<u8>,
    dimensions: Dimension,
    format: Option<Format>,
}

impl Display for SavedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = match self.format {
            Some(f) => &f.to_string(),
            None => "original",
        };
        write!(f, "{}_{}_{format}", self.url, self.dimensions)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl quickcheck::Arbitrary for Format {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // This works because Format is a C-style enum starting from 0
            // The size_hint here ensures we don't generate invalid discriminants
            match u8::arbitrary(g) % 3 {
                0 => Format::Avif,
                1 => Format::Png,
                _ => Format::Webp,
            }
        }
    }

    impl quickcheck::Arbitrary for Width {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self(u32::arbitrary(g))
        }
    }

    impl quickcheck::Arbitrary for Height {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self(u32::arbitrary(g))
        }
    }

    impl quickcheck::Arbitrary for Dimension {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Dimension(Option::arbitrary(g), Option::arbitrary(g))
        }
    }

    impl quickcheck::Arbitrary for SavedImage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                url: String::arbitrary(g),
                data: Vec::arbitrary(g),
                dimensions: Dimension::arbitrary(g),
                format: Option::arbitrary(g),
            }
        }
    }

    quickcheck::quickcheck! {
        fn saved_image_display(image: SavedImage) -> bool {
            !image.to_string().is_empty()
        }
    }

    #[test]
    fn download() -> Result<(), Error> {
        let url = "https://placehold.co/1x1";
        let file = download_image(url)?;

        assert!(!file.is_empty());
        Ok(())
    }

    #[test]
    fn convert_png() -> Result<(), Error> {
        let file = download_image("https://placehold.co/1x1.png")?;

        for format in [Format::Avif, Format::Webp, Format::Png].iter() {
            let new_file = convert_file(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    // BUG: avif to avif hangs infinitely
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

        for format in [Format::Png, Format::Avif, Format::Webp].iter() {
            let new_file = convert_file(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    #[test]
    fn saved_images_naming() {
        let all_defined = SavedImage {
            url: "localhost".to_string(),
            dimensions: Dimension(Some(Width(10)), Some(Height(10))),
            data: Vec::new(),
            format: Some(Format::Avif),
        };

        let no_size = SavedImage {
            url: "localhost".to_string(),
            dimensions: Dimension(None, None),
            data: Vec::new(),
            format: Some(Format::Avif),
        };

        let no_format = SavedImage {
            url: "localhost".to_string(),
            dimensions: Dimension(None, None),
            data: Vec::new(),
            format: None,
        };

        let no_height = SavedImage {
            url: "localhost".to_string(),
            dimensions: Dimension(Some(Width(100)), None),
            data: Vec::new(),
            format: Some(Format::Png),
        };

        let no_width = SavedImage {
            url: "localhost".to_string(),
            dimensions: Dimension(None, Some(Height(100))),
            data: Vec::new(),
            format: Some(Format::Webp),
        };

        assert_eq!("localhost_10_10_avif", &all_defined.to_string());
        assert_eq!("localhost_original_original_avif", &no_size.to_string());
        assert_eq!(
            "localhost_original_original_original",
            &no_format.to_string()
        );
        assert_eq!("localhost_100_original_png", &no_height.to_string());
        assert_eq!("localhost_original_100_webp", &no_width.to_string());
    }
}
