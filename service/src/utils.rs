use std::env;
use std::io::Cursor;

use image::DynamicImage;
use image::codecs::avif::AvifEncoder;
use rand::Rng;
use rand::distributions::Alphanumeric;
use s3::Bucket;
use s3::creds::Credentials;

use crate::error::AppError;

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
pub fn get_file_id() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(FILE_NAME_LENGTH)
        .map(char::from)
        .collect::<String>()
}

pub fn get_bucket() -> Result<Bucket, AppError> {
    let bucket_name = env::var("BUCKET_NAME").expect("Bucket name must be specified in the env");
    let region_name = env::var("REGION_NAME").expect("Region must be specified in the env");
    let endpoint = env::var("ENDPOINT").expect("Endpoint must be specified in the env");
    let access_key = env::var("ACCESS_KEY").expect("Access key must be specified in the env");
    let secret_key = env::var("SECRET_KEY").expect("Secret key must be specified in the env");

    let region = s3::Region::Custom {
        region: region_name,
        endpoint,
    };

    let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;

    Ok(Bucket::new(&bucket_name, region, credentials)?)
}

pub fn compress_image(
    original: &DynamicImage,
    dimensions: (u32, u32),
    speed: u8,
    quality: u8,
) -> Vec<u8> {
    let image = original.resize(
        dimensions.0,
        dimensions.1,
        image::imageops::FilterType::Triangle,
    );

    let mut bytes: Vec<u8> = Vec::new();

    let write = Cursor::new(&mut bytes);

    let encoder = AvifEncoder::new_with_speed_quality(write, speed, quality);
    image.write_with_encoder(encoder).unwrap();

    bytes
}
