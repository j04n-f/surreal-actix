use utoipa_actix_web::{scope, service_config::ServiceConfig};

mod controllers;
mod dto;
mod error;
mod middlewares;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(scope("/api/v1").configure(controllers::account::routes));
}
