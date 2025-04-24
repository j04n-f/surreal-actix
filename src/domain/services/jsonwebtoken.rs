use crate::domain::{
    error::AppResult,
    models::jsonwebtoken::{AccessToken, Claims},
};

pub trait JsonWebTokenService: 'static + Sync + Send {
    fn generate_token(&self, id: String) -> AppResult<AccessToken>;
    fn validate_token(&self, token: &str) -> AppResult<Claims>;
}
