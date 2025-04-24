use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::domain::models::account::{Account, CreateAccount};

#[derive(Debug, Deserialize)]
pub struct SurrealAccount {
    id: Thing,
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct SurrealAccountCreate {
    name: String,
    email: String,
    password: String,
}

impl From<CreateAccount> for SurrealAccountCreate {
    fn from(acc: CreateAccount) -> Self {
        SurrealAccountCreate {
            name: acc.name,
            email: acc.email,
            password: acc.password,
        }
    }
}

impl From<SurrealAccount> for Account {
    fn from(acc: SurrealAccount) -> Self {
        Account {
            id: acc.id.id.to_string(),
            name: acc.name,
            email: acc.email,
            password: acc.password,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SurrealCount {
    pub count: i64,
}
