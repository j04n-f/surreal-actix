use crate::domain::error::AppError;
use crate::domain::models::jsonwebtoken::Claims;
use crate::domain::services::jsonwebtoken::JsonWebTokenService;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use futures::future::{Ready, err, ok};
use std::sync::Arc;

#[derive(Debug)]
pub struct RequireJsonWebToken {
    #[allow(dead_code)]
    pub claims: Claims,
}

fn get_token(req: &HttpRequest) -> Result<String, AppError> {
    if let Some(cookie) = req.cookie("Authorization") {
        return Ok(cookie.value().to_string());
    }

    if let Some(header) = req.headers().get("Authorization") {
        return Ok(header
            .to_str()
            .map_err(|_| AppError::Unauthorized())?
            .trim_start_matches("Bearer")
            .to_string());
    }

    Err(AppError::Unauthorized())
}

impl FromRequest for RequireJsonWebToken {
    type Error = AppError;
    type Future = Ready<Result<RequireJsonWebToken, AppError>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(jsonwebtoken_service) =
            req.app_data::<web::Data<Arc<dyn JsonWebTokenService>>>()
        {
            return match get_token(req) {
                Ok(token) => match jsonwebtoken_service.validate_token(token.trim()) {
                    Ok(claims) => ok(RequireJsonWebToken { claims }),
                    Err(error) => err(error),
                },
                Err(error) => err(error),
            };
        }

        err(AppError::InternalError().trace("JsonWebTokenService is not defined"))
    }
}

#[cfg(test)]
mod tests {

    use actix_web::{
        App, HttpResponse, Responder,
        cookie::Cookie,
        http::StatusCode,
        test::{self, TestRequest},
        web,
    };

    use crate::services::jsonwebtoken::JsonWebTokenServiceImpl;
    use crate::tests::utils::crypto::generate_keypair;

    use super::*;

    async fn index(_: RequireJsonWebToken) -> impl Responder {
        HttpResponse::new(StatusCode::OK)
    }

    use rstest::*;

    #[fixture]
    fn jwt_service() -> Arc<dyn JsonWebTokenService> {
        Arc::new(JsonWebTokenServiceImpl::new(generate_keypair()))
    }

    enum Auth {
        Cookie,
        Header,
    }

    async fn send_req(
        name: &str,
        value: &str,
        auth: Auth,
        jsonwebtoken_service: Arc<dyn JsonWebTokenService>,
    ) -> StatusCode {
        let app = test::init_service(
            App::new()
                .route("/index", web::get().to(index))
                .app_data(web::Data::new(jsonwebtoken_service)),
        )
        .await;

        let mut req = TestRequest::get().uri("/index");

        match auth {
            Auth::Cookie => {
                req = req.cookie(
                    Cookie::build(name, value)
                        .http_only(true)
                        .secure(true)
                        .path("/")
                        .same_site(actix_web::cookie::SameSite::Strict)
                        .finish(),
                );
            }
            Auth::Header => {
                req = req.insert_header((name, format!("Bearer {value}")));
            }
        }

        let res = req.send_request(&app).await;

        res.status()
    }

    #[rstest]
    #[case::cookie(Auth::Cookie)]
    #[case::header(Auth::Header)]
    #[actix_web::test]
    async fn test_invalid_token(jwt_service: Arc<dyn JsonWebTokenService>, #[case] auth: Auth) {
        assert_eq!(
            send_req(
                "Authorization",
                "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9",
                auth,
                jwt_service
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[rstest]
    #[case::cookie(Auth::Cookie)]
    #[case::header(Auth::Header)]
    #[actix_web::test]
    async fn test_missing_token(jwt_service: Arc<dyn JsonWebTokenService>, #[case] auth: Auth) {
        assert_eq!(
            send_req(
                "Auth",
                "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9",
                auth,
                jwt_service
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[rstest]
    #[case::cookie(Auth::Cookie)]
    #[case::header(Auth::Header)]
    #[actix_web::test]
    async fn test_authorized_access(jwt_service: Arc<dyn JsonWebTokenService>, #[case] auth: Auth) {
        let access_token = jwt_service.generate_token("ajk".into()).unwrap();

        assert_eq!(
            send_req("Authorization", &access_token.token, auth, jwt_service).await,
            StatusCode::OK
        );
    }
}
