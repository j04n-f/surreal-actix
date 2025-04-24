use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub service: ServiceConfig,
    pub logging: LoggingConfig,
    pub surrealdb: SurrealDbConfig,
    pub jsonwebtoken: JsonWebTokenConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JsonWebTokenConfig {
    pub public_keyfile: String,
    pub private_keyfile: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ServiceConfig {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SurrealDbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
    pub migration: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Serialized::defaults(AppConfig {
                service: ServiceConfig {
                    name: "surreal-actix".to_string(),
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                },
                jsonwebtoken: JsonWebTokenConfig {
                    public_keyfile: "config/public_key.pem".to_string(),
                    private_keyfile: "config/private_key.pem".to_string(),
                },
                surrealdb: SurrealDbConfig {
                    host: "localhost".to_string(),
                    port: 8080,
                    username: "root".to_string(),
                    password: "root".to_string(),
                    namespace: "test".to_string(),
                    database: "test".to_string(),
                    migration: true,
                },
            }))
            .merge(Toml::file("config/default.toml"))
            .merge(Toml::file(format!(
                "config/{}.toml",
                std::env::var("RUST_ENV").unwrap_or("development".to_string())
            )))
            .merge(Env::prefixed("APP_").split("__"))
            .extract()
    }
}
