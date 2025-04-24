use actix_web::{
    HttpResponse, ResponseError,
    http::{StatusCode, header::ContentType},
    web::Json,
};

use actix_web::error::JsonPayloadError;

use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

use serde::Serialize;
use utoipa::ToSchema;

use serde_json::{Map, Value, to_string};

use argon2::password_hash::errors::Error::{self as Argon2Error, Password};

pub type AppResult<T> = core::result::Result<T, AppError>;

macro_rules! static_error {
    ($name:ident, $status:expr) => {
        #[allow(non_snake_case, missing_docs)]
        pub fn $name(message: impl ToString) -> AppError {
            AppError {
                message: message.to_string(),
                code: $status.as_u16(),
                trace: None,
            }
        }
    };

    ($name:ident, $status:expr, $default:expr) => {
        #[allow(non_snake_case, missing_docs)]
        pub fn $name() -> AppError {
            AppError {
                message: $default.to_string(),
                code: $status.as_u16(),
                trace: None,
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct AppError {
    pub message: String,
    pub code: u16,
    #[serde(skip)]
    pub trace: Option<String>,
}

#[rustfmt::skip]
pub mod message {
    pub static CONFLICT: &str = "Conflict with the current state of the resource";
    // pub static NOT_FOUND: &str = "The server cannot find the requested resource";
    pub static UNAUTHORIZED: &str = "The request was not successful because it lacks valid authentication credentials";
    pub static UNPROCESSABLE_ENTITY: &str = "The server was unable to process the request because it contains invalid data";
    pub static BAD_REQUEST: &str = "The server would not process the request due to something the server considered to be a client error";
    pub static INTERNAL_ERROR: &str = "The server encountered an unexpected condition that prevented it from fulfilling the request";
    pub static SERVICE_UNAVAILABLE: &str = "The server is not ready to handle the request";
}

#[rustfmt::skip]
impl AppError {
    // 1. Errors with Custom Message
    static_error!(Conflict, StatusCode::CONFLICT);
    static_error!(BadRequest, StatusCode::BAD_REQUEST);
    static_error!(UnprocessableEntity, StatusCode::UNPROCESSABLE_ENTITY);
    // static_error!(NotFound, StatusCode::NOT_FOUND);

    // 2. Errors with Default Message
    static_error!(Unauthorized, StatusCode::UNAUTHORIZED, message::UNAUTHORIZED);
    static_error!(InternalError, StatusCode::INTERNAL_SERVER_ERROR, message::INTERNAL_ERROR);
    static_error!(ServiceUnavailable, StatusCode::SERVICE_UNAVAILABLE, message::SERVICE_UNAVAILABLE);

    pub fn trace(self, message: &str) -> AppError {
        AppError {
            code: self.code,
            message: self.message,
            trace: Some(message.to_owned()),
        }
    }

    pub fn example_500() -> AppError {
        AppError::InternalError()
    }

    pub fn example_503() -> AppError {
        AppError::ServiceUnavailable()
    }

    pub fn example_401() -> AppError {
        AppError::Unauthorized()
    }

    pub fn example_422() -> AppError {
        AppError::UnprocessableEntity(message::UNPROCESSABLE_ENTITY)
    }

    pub fn example_400() -> AppError {
        AppError::BadRequest(message::BAD_REQUEST)
    }

    pub fn example_409() -> AppError {
        AppError::Conflict(message::CONFLICT)
    }

    // pub fn example_404() -> AppError {
    //     AppError::NotFound(message::NOT_FOUND)
    // }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .json(Json(self))
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap()
    }
}

impl From<surrealdb::Error> for AppError {
    fn from(error: surrealdb::Error) -> Self {
        AppError::InternalError().trace(&error.to_string())
    }
}

impl From<Argon2Error> for AppError {
    fn from(error: Argon2Error) -> Self {
        match error {
            Password => AppError::Unauthorized(),
            _ => AppError::InternalError().trace(&error.to_string()),
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let mut map = Map::new();

        for (_, field, error) in flatten_errors(&errors, None, None) {
            map.insert(field, Value::String(error.to_string()));
        }

        AppError::UnprocessableEntity(to_string(&map).unwrap())
    }
}

fn flatten_errors(
    errors: &ValidationErrors,
    path: Option<String>,
    indent: Option<u16>,
) -> Vec<(u16, String, &ValidationError)> {
    errors
        .errors()
        .iter()
        .flat_map(|(field, err)| {
            let indent = indent.unwrap_or(0);
            let actual_path = path
                .as_ref()
                .map(|path| [path.as_str(), field].join("."))
                .unwrap_or_else(|| field.to_string());
            match err {
                ValidationErrorsKind::Field(field_errors) => field_errors
                    .iter()
                    .map(|error| (indent, actual_path.clone(), error))
                    .collect::<Vec<_>>(),
                ValidationErrorsKind::List(list_error) => list_error
                    .iter()
                    .flat_map(|(index, errors)| {
                        let actual_path = format!("{}[{}]", actual_path.as_str(), index);
                        flatten_errors(errors, Some(actual_path), Some(indent + 1))
                    })
                    .collect::<Vec<_>>(),
                ValidationErrorsKind::Struct(struct_errors) => {
                    flatten_errors(struct_errors, Some(actual_path), Some(indent + 1))
                }
            }
        })
        .collect::<Vec<_>>()
}

impl From<JsonPayloadError> for AppError {
    fn from(error: JsonPayloadError) -> Self {
        AppError::BadRequest(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;
    use validator::Validate;

    #[derive(Debug, Validate, Serialize)]
    struct User {
        #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
        username: String,

        #[validate(email(message = "Invalid email format"))]
        email: String,

        #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
        password: String,

        #[validate(nested)]
        profile: Profile,

        #[validate(nested)]
        addresses: Vec<Address>,
    }

    #[derive(Debug, Validate, Serialize)]
    struct Profile {
        #[validate(range(min = 0, max = 150, message = "Age must be between 0 and 150"))]
        age: i32,
    }

    #[derive(Debug, Validate, Serialize)]
    struct Address {
        #[validate(length(min = 5, message = "Street must be at least 5 characters long"))]
        street: String,
    }

    #[test]
    fn test_validation_error_parsing() {
        let user = User {
            username: "ab".to_string(),
            email: "invalid-email".to_string(),
            password: "123".to_string(),
            profile: Profile { age: 200 },
            addresses: vec![Address {
                street: "123".to_string(),
            }],
        };

        let app_error: AppError = user.validate().unwrap_err().into();

        let message = serde_json::from_str::<serde_json::Value>(&app_error.message).unwrap();

        assert_eq!(
            message,
            json!({
                "addresses[0].street": "Street must be at least 5 characters long",
                "email": "Invalid email format",
                "password": "Password must be at least 8 characters long",
                "profile.age": "Age must be between 0 and 150",
                "username": "Username must be at least 3 characters long"
            })
        );
    }
}
