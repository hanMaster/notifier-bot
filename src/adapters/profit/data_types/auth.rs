use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
}