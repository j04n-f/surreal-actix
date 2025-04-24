mod api;
mod app;
mod config;
mod container;
mod domain;
mod infrastructure;
mod opentelemetry;
mod services;

use config::AppConfig;
use container::Container;
use infrastructure::databases::surrealdb;
use services::jsonwebtoken::KeyPair;

use actix_web::HttpServer;
use include_dir::{Dir, include_dir};
use std::fs;
use std::sync::Arc;
use surrealdb_migrations::MigrationRunner;
use thiserror::Error;

#[cfg(test)]
mod tests;

const MIGRATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migration");

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] ::surrealdb::Error),
    #[error(transparent)]
    Configuration(#[from] figment::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Migration(String),
    #[error(transparent)]
    OTel(#[from] opentelemetry::OTelError),
    #[error(transparent)]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    #[error("{0}: {1}")]
    ReadKey(String, String),
}

async fn run() -> Result<(), AppError> {
    let config = AppConfig::load()?;

    let conn = surrealdb::connect(&config.surrealdb).await?;

    if config.surrealdb.migration {
        MigrationRunner::new(&conn)
            .load_files(&MIGRATIONS_DIR)
            .up()
            .await
            .map_err(|err| AppError::Migration(err.to_string()))?;
    }

    let provider = opentelemetry::configure(&config.service, &config.logging)?;

    let private_key = read_key(&config.jsonwebtoken.private_keyfile)?;
    let public_key = read_key(&config.jsonwebtoken.public_keyfile)?;

    let keys = KeyPair::from_rsa_pem(private_key, public_key)?;

    let container = Arc::new(Container::new(conn, keys));

    HttpServer::new(move || app::create(Arc::clone(&container)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    opentelemetry::shutdown(provider)?;

    Ok(())
}

#[actix_web::main]
async fn main() {
    if let Err(err) = run().await {
        panic!("{err}");
    }
}

fn read_key(keyfile: &str) -> Result<Vec<u8>, AppError> {
    fs::read(keyfile).map_err(|err| AppError::ReadKey(err.to_string(), keyfile.to_string()))
}
