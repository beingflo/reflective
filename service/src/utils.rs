use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::error::AppError;

const AUTH_TOKEN_LENGTH: usize = 64;
const OBJECT_NAME_LENGTH: usize = 32;

/// Get a secure token for session tokens
pub fn get_auth_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get a random object name for s3 bucket
pub fn get_object_name() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(OBJECT_NAME_LENGTH)
        .map(char::from)
        .collect::<String>()
}

pub fn compress_image(
    original: &DynamicImage,
    dimensions: (u32, u32),
    quality: u8,
) -> Result<Vec<u8>, AppError> {
    let image = original.resize(
        dimensions.0,
        dimensions.1,
        image::imageops::FilterType::Triangle,
    );

    let mut bytes: Vec<u8> = Vec::new();

    let write = Cursor::new(&mut bytes);

    let encoder = JpegEncoder::new_with_quality(write, quality);
    image.write_with_encoder(encoder)?;

    Ok(bytes)
}
