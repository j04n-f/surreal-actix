use async_trait::async_trait;

use crate::domain::models::account::{Account, CreateAccount};

use super::repository::RepositoryResult;

#[derive(Debug, Clone)]
pub enum FindByCol {
    Email(String),
}

impl FindByCol {
    pub fn value(self) -> String {
        match self {
            Self::Email(email) => email,
        }
    }
}

impl std::fmt::Display for FindByCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email(_) => write!(f, "email"),
        }
    }
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn is_account(&self, email: &str) -> RepositoryResult<bool>;
    async fn signup(&self, new_account: CreateAccount) -> RepositoryResult<Account>;
    async fn find_one(&self, column: FindByCol) -> RepositoryResult<Option<Account>>;
}
