use actix_web::cookie::Cookie;
use actix_web::http::StatusCode;
use rstest::*;
use serde::Deserialize;
use serde_json::json;

use crate::tests::utils::seed::seed_account;
use crate::tests::{Error, TestContext, context};

use crate::app;
use actix_web::test;
use actix_web::test::TestRequest;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[allow(dead_code)]
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessToken {
    token: String,
    expires_at: i64,
}

#[rstest]
#[awt]
#[actix_web::test]
async fn test_success_signup(#[future] context: TestContext) {
    let app = test::init_service(app::create(context.container)).await;

    let res = TestRequest::post()
        .uri("/api/v1/signup")
        .set_json(json!({
                "name": "New Account",
                "email": "new_account@email.com",
                "password": "stR0ngP4ssw0rd!",
        }))
        .send_request(&app)
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let acc: Account = test::read_body_json(res).await;

    assert_eq!(acc.name, "New Account");
    assert_eq!(acc.email, "new_account@email.com");

    let _ = context.db.container.stop().await;
}

#[rstest]
#[awt]
#[actix_web::test]
async fn test_signup_twice(#[future] context: TestContext) {
    let app = test::init_service(app::create(context.container)).await;

    let account = seed_account(&context.db.connection).await;

    let res = TestRequest::post()
        .uri("/api/v1/signup")
        .set_json(json!({
            "name": account.name,
            "email": account.email,
            "password": account.password,
        }))
        .send_request(&app)
        .await;

    assert_eq!(res.status(), StatusCode::CONFLICT);

    let err: Error = test::read_body_json(res).await;

    assert_eq!(err.code, 409);
    assert_eq!(err.message, "Account already exists");

    let _ = context.db.container.stop().await;
}

#[rstest]
#[awt]
#[actix_web::test]
async fn test_success_signin(#[future] context: TestContext) {
    let app = test::init_service(app::create(context.container)).await;

    let account = seed_account(&context.db.connection).await;

    let res = TestRequest::post()
        .uri("/api/v1/signin")
        .set_json(json!({
            "email": account.email,
            "password": account.password,
        }))
        .send_request(&app)
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let headers = res.headers().clone();
    let header = headers.get("set-cookie").unwrap();
    let cookie = header.to_str().unwrap();

    let cookie = Cookie::parse_encoded(cookie.to_owned()).unwrap();

    let access_token: AccessToken = test::read_body_json(res).await;

    assert_eq!(cookie.name(), "Authorization");
    assert_eq!(cookie.value(), access_token.token);
    assert!(access_token.expires_at > 0);
    assert_eq!(
        cookie
            .expires()
            .unwrap()
            .datetime()
            .unwrap()
            .unix_timestamp(),
        access_token.expires_at
    );

    let _ = context.db.container.stop().await;
}

#[rstest]
#[case::invalid_email("fake_account@email.com", "stR0ngP4ssw0rd!")]
#[case::invalid_password("test_account@email.com", "p4ssw0rd")]
#[awt]
#[actix_web::test]
async fn test_invalid_signin(
    #[future] context: TestContext,
    #[case] email: String,
    #[case] password: String,
) {
    let app = test::init_service(app::create(context.container)).await;

    let res = TestRequest::post()
        .uri("/api/v1/signin")
        .set_json(json!({
            "email": email,
            "password": password,
        }))
        .send_request(&app)
        .await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let err: Error = test::read_body_json(res).await;
    assert_eq!(err.code, 401);
    assert_eq!(
        err.message,
        "The request was not successful because it lacks valid authentication credentials"
    );

    let _ = context.db.container.stop().await;
}
