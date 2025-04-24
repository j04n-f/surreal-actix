mod account;

pub mod utils;

use ::surrealdb::{Surreal, engine::remote::ws::Client};

use crate::tests::utils::crypto::generate_keypair;
use std::sync::Arc;

use serde::Deserialize;
use surrealdb_migrations::MigrationRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::{
    surrealdb::{SURREALDB_PORT, SurrealDb},
    testcontainers::runners::AsyncRunner,
};

use crate::{MIGRATIONS_DIR, infrastructure::databases::surrealdb};
use crate::{config::AppConfig, container::Container};

use actix_http::Request;
use actix_web::cookie::Cookie;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceResponse},
    test::TestRequest,
};

use serde_json::json;

use rstest::*;

struct Database {
    pub connection: Surreal<Client>,
    pub container: ContainerAsync<SurrealDb>,
}

pub struct TestContext {
    pub db: Database,
    pub container: Arc<Container>,
}

#[fixture]
async fn context() -> TestContext {
    let db_container = SurrealDb::default()
        .with_tag("latest")
        .start()
        .await
        .unwrap();

    let mut config = AppConfig::load().unwrap();

    config.surrealdb.port = db_container
        .get_host_port_ipv4(SURREALDB_PORT)
        .await
        .unwrap();

    let db_connection = surrealdb::connect(&config.surrealdb).await.unwrap();

    let _ = MigrationRunner::new(&db_connection)
        .load_files(&MIGRATIONS_DIR)
        .up()
        .await;

    let keys = generate_keypair();

    let db = Database {
        connection: db_connection.clone(),
        container: db_container,
    };

    let container = Arc::new(Container::new(db_connection, keys));

    TestContext { db, container }
}

async fn request_cookie<'a, S, B>(app: &'a S, email: &'a str, password: &'a str) -> Cookie<'a>
where
    S: Service<Request, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    let res = TestRequest::post()
        .uri("/api/v1/signin")
        .set_json(json!({
            "email": email,
            "password": password,
        }))
        .send_request(&app)
        .await;

    let headers = res.headers().clone();
    let header = headers.get("set-cookie").unwrap();
    let cookie = header.to_str().unwrap();

    Cookie::parse_encoded(cookie.to_owned()).unwrap()
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Error {
    code: u16,
    message: String,
}
