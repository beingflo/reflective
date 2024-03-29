use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use rand::distributions::Alphanumeric;
use rand::Rng;
use s3::creds::Credentials;
use s3::Bucket;

use crate::error::AppError;
use crate::user::S3Data;

const AUTH_TOKEN_LENGTH: usize = 64;
const FILE_NAME_LENGTH: usize = 32;

/// Get a secure token for session tokens
pub fn get_auth_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get a random file name
pub fn get_file_name() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(FILE_NAME_LENGTH)
        .map(char::from)
        .collect::<String>()
}

pub fn get_bucket(config: S3Data) -> Result<Bucket, AppError> {
    let bucket_name = config.bucket;

    let region = s3::Region::Custom {
        region: config.region,
        endpoint: config.endpoint,
    };

    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )?;

    Ok(Bucket::new(&bucket_name, region, credentials)?)
}

pub fn format_filename(filename: &str, quality: &str) -> String {
    format!("{}-{}", filename, quality)
}

pub fn compress_image(original: &DynamicImage, size: u32, quality: u8) -> Vec<u8> {
    let image = original.resize(size, size, image::imageops::FilterType::Triangle);

    let mut bytes: Vec<u8> = Vec::new();

    let write = Cursor::new(&mut bytes);

    let encoder = JpegEncoder::new_with_quality(write, quality);
    image.write_with_encoder(encoder).unwrap();

    bytes
}
