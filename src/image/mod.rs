use std::{fmt::Display, io::Cursor};

use image::{guess_format, load_from_memory};
use serde::Deserialize;

use crate::error::Error;

/// `download` is a little helper function that takes a url and returns
/// a Vec<u8> with its contents.
/// # Errors
/// - When performing ureq request
/// - When saving the file to a buffer
pub fn download(url: &str) -> Result<Vec<u8>, Error> {
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

/// `convert` converts the `Format` of a given image from its underliying format
/// to the given one.
fn convert(file: &[u8], to: Format) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(file)?;
    let mut converted_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut converted_img, to.into())?;
    Ok(converted_img.get_ref().clone())
}

/// `resize_image` scales _*down*_ the given image.
fn resize(file: &[u8], d: &Dimension) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(file)?;
    let (width, height) = match d {
        Dimension(Some(Width(w)), Some(Height(h))) => (*w, *h),
        Dimension(Some(Width(w)), None) => (*w, img.height()),
        Dimension(None, Some(Height(h))) => (img.width(), *h),
        Dimension(None, None) => return Err(Error::ResizeEmptyDimension),
    };
    let img = img.thumbnail(width, height);
    let mut resized_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut resized_img, guess_format(file)?)?;
    Ok(resized_img.get_ref().clone())
}

pub enum Operation {
    Convert(Format),
    Resize(Dimension),
}

/// `transform` performs all needed operations on an image (byte slice)
/// # Errors
/// - When trying to guess the image format
pub fn transform(file: &[u8], op: &[Operation]) -> Result<Vec<u8>, Error> {
    op.iter().try_fold(Vec::with_capacity(0), |acc, o| {
        // don't use accumulator on our first pass
        // and check that the passed value is a valid image
        // this results in a small overhead
        // (compared to just checking for a non-empty file)
        // in this case its probably an overkill.
        guess_format(file)?;
        let acc = if acc.is_empty() { file } else { &acc };

        match o {
            Operation::Convert(f) => convert(acc, *f),
            Operation::Resize(s) => resize(acc, s),
        }
    })
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Avif,
    Png,
    Webp,
}

impl Format {
    #[must_use]
    pub const fn content_type(&self) -> &str {
        match self {
            Self::Avif => "image/avif",
            Self::Webp => "image/webp",
            Self::Png => "image/png",
        }
    }
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
            Format::Avif => Self::Avif,
            Format::Png => Self::Png,
            Format::Webp => Self::WebP,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Width(u32);
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Height(u32);
#[derive(Clone, Debug, Deserialize)]
pub struct Dimension(pub Option<Width>, pub Option<Height>);

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

/// `Saved` holds the info about an image saved in cache.
#[derive(Clone, Debug)]
pub struct Saved {
    pub url: String,
    pub dimensions: Dimension,
    pub format: Option<Format>,
}

//TODO: implement fromstr for savedimage to revert this process?
impl Display for Saved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = self
            .format
            .map_or("original".to_string(), |f| f.to_string());
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

    impl quickcheck::Arbitrary for Saved {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                url: String::arbitrary(g),
                dimensions: Dimension::arbitrary(g),
                format: Option::arbitrary(g),
            }
        }
    }

    quickcheck::quickcheck! {
        fn saved_image_display(image: Saved) -> bool {
            !image.to_string().is_empty()
        }
    }

    #[test]
    fn download_image() -> Result<(), Error> {
        let url = "https://placehold.co/1x1";
        let file = download(url)?;

        assert!(!file.is_empty());
        Ok(())
    }

    #[test]
    fn convert_png() -> Result<(), Error> {
        let file = download("https://placehold.co/1x1.png")?;

        for format in [Format::Avif, Format::Webp, Format::Png].iter() {
            let new_file = convert(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    // BUG: avif to avif hangs infinitely
    #[test]
    fn convert_avif() -> Result<(), Error> {
        let file = download("https://raw.githubusercontent.com/link-u/avif-sample-images/refs/heads/master/kimono.avif")?;

        for format in [Format::Png, Format::Webp].iter() {
            let new_file = convert(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    #[test]
    fn convert_webp() -> Result<(), Error> {
        let file = download("https://placehold.co/1x1.webp")?;

        for format in [Format::Png, Format::Avif, Format::Webp].iter() {
            let new_file = convert(&file, *format);
            assert!(new_file.is_ok());
        }

        Ok(())
    }
    #[test]
    fn saved_images_naming() {
        let all_defined = Saved {
            url: "localhost".to_string(),
            dimensions: Dimension(Some(Width(10)), Some(Height(10))),
            format: Some(Format::Avif),
        };

        let no_size = Saved {
            url: "localhost".to_string(),
            dimensions: Dimension(None, None),
            format: Some(Format::Avif),
        };

        let no_format = Saved {
            url: "localhost".to_string(),
            dimensions: Dimension(None, None),
            format: None,
        };

        let no_height = Saved {
            url: "localhost".to_string(),
            dimensions: Dimension(Some(Width(100)), None),
            format: Some(Format::Png),
        };

        let no_width = Saved {
            url: "localhost".to_string(),
            dimensions: Dimension(None, Some(Height(100))),
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
