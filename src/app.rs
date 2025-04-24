use actix_web::{
    App, HttpMessage,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    http::header,
    middleware::{Next, from_fn},
    web,
};

use tracing_actix_web::{RequestId, TracingLogger};

use actix_cors::Cors;

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::{Components, OpenApi, Server};
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

use crate::api;
use crate::container::Container;

use std::sync::Arc;

pub fn create(
    container: Arc<Container>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .into_utoipa_app()
        .openapi(docs())
        .configure(api::routes)
        .openapi_service(|api| {
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
        })
        .into_app()
        .wrap(TracingLogger::default())
        .wrap(cors())
        .wrap(from_fn(request_headers))
        .app_data(web::Data::new(container.account_service.clone()))
        .app_data(web::Data::new(container.jsonwebtoken_service.clone()))
}

fn cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:8080")
        .allowed_origin("http://localhost:8080")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(&[header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allowed_header(header::CONTENT_TYPE)
        .block_on_origin_mismatch(false)
        .max_age(3600)
}

async fn request_headers(
    req: ServiceRequest,
    svc: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let request_id = req.extensions().get::<RequestId>().copied();
    let mut res = svc.call(req).await?;

    if let Some(request_id) = request_id {
        res.headers_mut().insert(
            header::HeaderName::from_static("x-request-id"),
            header::HeaderValue::from_str(&request_id.to_string()).unwrap(),
        );
    }
    Ok(res)
}

pub fn docs() -> OpenApi {
    let mut openapi = OpenApi::default();

    openapi.info.title = String::from("Surreal-Actix API");
    openapi.info.description = Some(String::from("API Docs for Surreal-Actix"));
    openapi.info.version = String::from(env!("CARGO_PKG_VERSION"));

    openapi.servers = Some(servers());
    openapi.components = Some(components());

    openapi
}

fn servers() -> Vec<Server> {
    vec![server("http://localhost:8080", "Localhost")]
}

fn server(url: &str, description: &str) -> Server {
    Server::builder()
        .description(Some(description.to_owned()))
        .url(url.to_owned())
        .build()
}

fn components() -> Components {
    Components::builder()
        .security_scheme(
            "jsonwebtoken",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
        .build()
}
