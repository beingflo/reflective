use rand::distributions::Alphanumeric;
use rand::Rng;

const AUTH_TOKEN_LENGTH: usize = 64;
const FILE_NAME_LENGTH: usize = 16;

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
