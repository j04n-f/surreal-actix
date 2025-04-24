use async_trait::async_trait;

use crate::domain::error::AppResult;
use crate::domain::models::account::{Account, CreateAccount, Credentials};

#[async_trait]
pub trait AccountService: 'static + Sync + Send {
    async fn signin(&self, credentials: Credentials) -> AppResult<Account>;
    async fn signup(&self, mut new_account: CreateAccount) -> AppResult<Account>;
}
