use serde::{Deserialize, Serialize};

pub struct AccessToken {
    pub token: String,
    pub expiration: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}
