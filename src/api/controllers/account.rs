use std::sync::Arc;

use crate::api::error::ApiResult;
use crate::api::middlewares::validate::Json;
use crate::domain::error::AppError;
use crate::domain::services::account::AccountService;

use crate::api::dto::account::{AccessTokenDTO, AccountDTO, CreateAccountDTO, CredentialsDTO};
use crate::domain::services::jsonwebtoken::JsonWebTokenService;

use actix_web::{
    HttpResponse,
    cookie::time::OffsetDateTime,
    cookie::{Cookie, SameSite},
    post,
    web::Data as State,
};

use utoipa_actix_web::service_config::ServiceConfig;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(signup).service(signin);
}

#[utoipa::path(
    responses(
        (status = 200, body = AccountDTO, description = "Account Created"),
        (status = 400, body = AppError, example = json!(AppError::example_400())),
        (status = 401, body = AppError, example = json!(AppError::example_401())),
        (status = 409, body = AppError, example = json!(AppError::example_409())),
        (status = 422, body = AppError, example = json!(AppError::example_422())),
        (status = 500, body = AppError, example = json!(AppError::example_500())),
        (status = 503, body = AppError, example = json!(AppError::example_503()))
    ),
    request_body = CreateAccountDTO,
    tag = "Account",
)]
#[post("/signup")]
pub async fn signup(
    payload: Json<CreateAccountDTO>,
    account_service: State<Arc<dyn AccountService>>,
) -> ApiResult {
    let account_dto = payload.into_inner();

    let created_account = account_service.signup(account_dto.into()).await?;

    Ok(HttpResponse::Ok().json(AccountDTO::from(created_account)))
}

#[utoipa::path(
    responses(
        (status = 200, body = AccessTokenDTO),
        (status = 400, body = AppError, example = json!(AppError::example_400())),
        (status = 401, body = AppError, example = json!(AppError::example_401())),
        (status = 500, body = AppError, example = json!(AppError::example_500())),
        (status = 503, body = AppError, example = json!(AppError::example_503()))
    ),
    request_body = CredentialsDTO,
    tag = "Account"
)]
#[post("/signin")]
pub async fn signin(
    payload: Json<CredentialsDTO>,
    account_service: State<Arc<dyn AccountService>>,
    jsonwebtoken_service: State<Arc<dyn JsonWebTokenService>>,
) -> ApiResult {
    let credentials_dto = payload.into_inner();

    let account = account_service.signin(credentials_dto.into()).await?;

    let access_token = jsonwebtoken_service.generate_token(account.id)?;

    let cookie = Cookie::build("Authorization", &access_token.token)
        .http_only(true)
        .secure(true)
        .path("/api")
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::from_unix_timestamp(access_token.expiration).unwrap())
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).json(AccessTokenDTO {
        token: access_token.token,
        expires_at: access_token.expiration,
    }))
}

#[cfg(test)]
mod tests {

    use actix_web::{
        App,
        dev::ServiceResponse,
        http::StatusCode,
        test::{self, TestRequest},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use utoipa_actix_web::AppExt;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Error {
        code: u16,
        message: String,
    }

    async fn signup(data: Value) -> ServiceResponse {
        let app =
            test::init_service(App::new().into_utoipa_app().configure(routes).into_app()).await;

        TestRequest::post()
            .uri("/signup")
            .set_json(data)
            .send_request(&app)
            .await
    }

    #[actix_web::test]
    async fn test_signup_invalid_email_format() {
        let payload = json!({
            "name": "Invalid Account",
            "email": "not-an-email",
            "password": "stR0ngP4ssw0rd!"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 422);
        assert!(err.message.contains("Invalid email format"));
    }

    #[actix_web::test]
    async fn test_signup_weak_password() {
        let payload = json!({
            "name": "Invalid Account",
            "email": "invalid@email.com",
            "password": "weak"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 422);
        assert!(
            err.message
                .contains("Password must contain between 8 and 72 characters")
        );
    }

    #[actix_web::test]
    async fn test_signup_missing_name() {
        let payload = json!({
            "email": "valid@email.com",
            "password": "stR0ngP4ssw0rd!"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 400);
    }

    #[actix_web::test]
    async fn test_signup_missing_email() {
        let payload = json!({
            "name": "Valid Name",
            "password": "stR0ngP4ssw0rd!"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 400);
    }

    #[actix_web::test]
    async fn test_signup_missing_password() {
        let payload = json!({
            "name": "Valid Name",
            "email": "valid@email.com"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 400);
    }

    #[actix_web::test]
    async fn test_signup_short_name() {
        let payload = json!({
            "name": "Ab",  // Name must have at least 3 characters
            "email": "valid@email.com",
            "password": "stR0ngP4ssw0rd!"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 422);
        assert!(err.message.contains("Name must have at least 3 characters"));
    }

    #[actix_web::test]
    async fn test_signup_password_without_special_char() {
        let payload = json!({
            "name": "Valid Name",
            "email": "valid@email.com",
            "password": "Password123"  // Missing special character
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 422);
        assert!(err.message.contains("Password must contain at least one uppercase letter, one lowercase letter, one digit and one special character"));
    }

    #[actix_web::test]
    async fn test_signup_email_too_long() {
        // Create a very long email
        let long_prefix = "a".repeat(250);
        let long_email = format!("{}@example.com", long_prefix);

        let payload = json!({
            "name": "Valid Name",
            "email": long_email,
            "password": "stR0ngP4ssw0rd!"
        });

        let res = signup(payload).await;

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let err: Error = test::read_body_json(res).await;
        assert_eq!(err.code, 422);
        assert!(
            err.message
                .contains("Email must contain between 3 and 255 characters")
        );
    }
}
