/// Get a secure token for session tokens
pub fn get_auth_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}