use crate::api::dto::validation::{is_email, is_name, is_password};
use crate::domain::models::account::CreateAccount;
use crate::domain::models::account::{Account, Credentials};
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountDTO {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateAccountDTO {
    #[validate(custom(function = "is_name"))]
    #[schema(examples("your_name"))]
    pub name: String,

    #[validate(custom(function = "is_email"))]
    #[schema(examples("your@email.com"))]
    pub email: String,

    #[validate(custom(function = "is_password"))]
    #[schema(examples("stR0ngP4ssw0rd!"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CredentialsDTO {
    #[validate(custom(function = "is_email"))]
    #[schema(examples("your@email.com"))]
    pub email: String,

    #[schema(examples("stR0ngP4ssw0rd!"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccessTokenDTO {
    #[schema(examples("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"))]
    pub token: String,
    #[schema(examples(1385903))]
    pub expires_at: i64,
}

impl From<Account> for AccountDTO {
    fn from(val: Account) -> Self {
        AccountDTO {
            id: val.id,
            name: val.name,
            email: val.email,
        }
    }
}

impl From<CreateAccountDTO> for CreateAccount {
    fn from(create_account: CreateAccountDTO) -> Self {
        CreateAccount {
            name: create_account.name,
            email: create_account.email,
            password: create_account.password,
        }
    }
}

impl From<CredentialsDTO> for Credentials {
    fn from(credentials: CredentialsDTO) -> Self {
        Credentials {
            email: credentials.email,
            password: credentials.password,
        }
    }
}
