use crate::domain::error::{AppError, AppResult};
use crate::domain::models::jsonwebtoken::{AccessToken, Claims};
use crate::domain::services::jsonwebtoken::JsonWebTokenService;
use chrono::Utc;
use jsonwebtoken::errors::{Error as JsonWebTokenError, ErrorKind};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

#[derive(Clone)]
pub struct KeyPair {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl KeyPair {
    pub fn from_rsa_pem(
        private_key: Vec<u8>,
        public_key: Vec<u8>,
    ) -> Result<Self, JsonWebTokenError> {
        Ok(KeyPair {
            encoding: EncodingKey::from_rsa_pem(&private_key)?,
            decoding: DecodingKey::from_rsa_pem(&public_key)?,
        })
    }
}

pub struct JsonWebTokenServiceImpl {
    keys: KeyPair,
}

impl JsonWebTokenServiceImpl {
    pub fn new(keys: KeyPair) -> Self {
        JsonWebTokenServiceImpl { keys }
    }
}

impl JsonWebTokenService for JsonWebTokenServiceImpl {
    fn generate_token(&self, id: String) -> AppResult<AccessToken> {
        let now = Utc::now();

        let expiration = now
            .checked_add_signed(chrono::Duration::hours(1))
            .unwrap()
            .timestamp();

        let iat = now.timestamp();

        let claims = Claims {
            sub: id,
            exp: expiration as usize,
            iat: iat as usize,
        };

        let header = Header::new(Algorithm::RS256);

        let token = encode(&header, &claims, &self.keys.encoding)
            .map_err(|err| AppError::InternalError().trace(&err.to_string()))?;

        Ok(AccessToken { token, expiration })
    }

    fn validate_token(&self, token: &str) -> AppResult<Claims> {
        match decode::<Claims>(
            token,
            &self.keys.decoding,
            &Validation::new(Algorithm::RS256),
        ) {
            Ok(token) => Ok(token.claims),
            Err(error) => match error.kind() {
                ErrorKind::ExpiredSignature
                | ErrorKind::InvalidToken
                | ErrorKind::InvalidIssuer => Err(AppError::Unauthorized()),
                _ => Err(AppError::InternalError().trace(&format!("{error:?}"))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::crypto::generate_keypair;

    use rstest::*;

    #[fixture]
    #[once]
    fn jwt_service() -> JsonWebTokenServiceImpl {
        JsonWebTokenServiceImpl::new(generate_keypair())
    }

    #[fixture]
    fn access_token(jwt_service: &JsonWebTokenServiceImpl) -> AccessToken {
        jwt_service.generate_token("test_id".to_string()).unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_token_expiration(access_token: AccessToken) {
        assert!(access_token.expiration - Utc::now().timestamp() < (60 * 60 * 1000 + 200));
    }

    #[rstest]
    #[tokio::test]
    async fn test_token_validation(
        jwt_service: &JsonWebTokenServiceImpl,
        access_token: AccessToken,
    ) {
        let claims = jwt_service.validate_token(&access_token.token).unwrap();
        assert_eq!(claims.sub, "test_id");
    }

    #[rstest]
    #[tokio::test]
    async fn test_invalid_token(jwt_service: &JsonWebTokenServiceImpl) {
        assert_eq!(
            jwt_service.validate_token("invalidtoken").unwrap_err(),
            AppError::Unauthorized()
        );
    }
}
