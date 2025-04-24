use actix_web::HttpResponse;

use crate::domain::error::AppResult;

pub type ApiResult = AppResult<HttpResponse>;
