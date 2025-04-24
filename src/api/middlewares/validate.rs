use core::fmt::Debug;
use std::ops::Deref;

use crate::domain::error::AppError;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::{JsonBody, Payload};
use futures::future::{FutureExt, LocalBoxFuture};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = AppError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(
            req,
            payload,
            Some(&|mime| mime == mime::APPLICATION_JSON),
            true,
        )
        .limit(32768)
        .map(|res: Result<T, _>| match res {
            Ok(payload) => payload
                .validate()
                .map(|_| Json(payload))
                .map_err(AppError::from),
            Err(err) => Err(AppError::from(err)),
        })
        .boxed_local()
    }
}

#[cfg(test)]
mod tests {

    use actix_web::{
        App, HttpResponse, Responder,
        http::{StatusCode, header::ContentType},
        test::{self, TestRequest},
        web,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Validate)]
    struct UserDTO {
        #[validate(length(
            min = 3,
            max = 50,
            message = "Name must be between 3 and 50 characters"
        ))]
        name: String,
        #[validate(email(message = "Invalid email format"))]
        email: String,
    }

    #[derive(Deserialize)]
    struct Error {
        code: u16,
        message: String,
    }

    async fn index(data: Json<UserDTO>) -> impl Responder {
        HttpResponse::Ok().json(data.0)
    }

    async fn send_req<T: DeserializeOwned>(data: &str) -> (StatusCode, T) {
        let app = test::init_service(App::new().route("/index", web::post().to(index))).await;

        let res = TestRequest::post()
            .uri("/index")
            .set_payload(data.to_string())
            .insert_header(ContentType::json())
            .send_request(&app)
            .await;

        let status = res.status();
        let body: T = test::read_body_json(res).await;

        (status, body)
    }

    #[actix_web::test]
    async fn test_valid_data() {
        let data = UserDTO {
            name: "new_user".to_string(),
            email: "new_user@spacecraft.com".to_string(),
        };

        let (status, body) = send_req::<UserDTO>(&json!(data).to_string()).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, data);
    }

    #[actix_web::test]
    async fn test_invalid_field_value() {
        let (status, err) =
            send_req::<Error>("{ \"email\": \"spacecraft.com\", \"name\": \"new_user\" }").await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.message, "{\"email\":\"Invalid email format\"}");
    }

    #[actix_web::test]
    async fn test_multiple_invalid_field_values() {
        let (status, err) =
            send_req::<Error>("{ \"email\": \"spacecraft.com\", \"name\": \"\" }").await;

        let message = serde_json::from_str::<serde_json::Value>(&err.message).unwrap();

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            message,
            json!({
                "name": "Name must be between 3 and 50 characters",
                "email": "Invalid email format"
            })
        );
    }

    #[actix_web::test]
    async fn test_invalid_field_type() {
        let (status, err) = send_req::<Error>("{ \"email\": 50, \"name\": \"new_user\" }").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            err.message,
            "Json deserialize error: invalid type: integer `50`, expected a string at line 1 column 13"
        );
    }

    #[actix_web::test]
    async fn test_missing_field() {
        let (status, err) = send_req::<Error>("{ \"name\": \"new_user\" }").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            err.message,
            "Json deserialize error: missing field `email` at line 1 column 22"
        );
    }

    #[actix_web::test]
    async fn test_malformed_data() {
        let (status, err) = send_req::<Error>("{ \"email\": }").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            err.message,
            "Json deserialize error: expected value at line 1 column 12"
        );
    }

    #[actix_web::test]
    async fn test_empty_data() {
        let (status, err) = send_req::<Error>("").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            err.message,
            "Json deserialize error: EOF while parsing a value at line 1 column 0"
        );
    }
}
