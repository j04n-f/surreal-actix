use std::sync::Arc;

use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

use crate::domain::models::account::{Account, CreateAccount};
use crate::domain::repositories::account::{AccountRepository, FindByCol};
use crate::domain::repositories::repository::RepositoryResult;
use crate::infrastructure::models::account::{SurrealAccount, SurrealAccountCreate, SurrealCount};

pub struct AccountRepositoryImpl {
    db: Arc<Surreal<Client>>,
}

impl AccountRepositoryImpl {
    pub fn new(db: Arc<Surreal<Client>>) -> Self {
        Self { db }
    }
}

const ACCOUNT: &str = "account";

#[async_trait]
impl AccountRepository for AccountRepositoryImpl {
    async fn signup(&self, new_account: CreateAccount) -> RepositoryResult<Account> {
        let account: SurrealAccount = self
            .db
            .create(ACCOUNT)
            .content(SurrealAccountCreate::from(new_account))
            .await?
            .unwrap();

        Ok(account.into())
    }

    async fn is_account(&self, email: &str) -> RepositoryResult<bool> {
        let mut res = self
            .db
            .query("(SELECT count() FROM type::table($table) WHERE email = type::string($email))[0] or { count: 0 }")
            .bind(("table", ACCOUNT))
            .bind(("email", email.to_owned()))
            .await?;

        let counter = res.take::<Option<SurrealCount>>(0)?.unwrap();

        Ok(counter.count > 0)
    }

    async fn find_one(&self, column: FindByCol) -> RepositoryResult<Option<Account>> {
        let account: Option<SurrealAccount> = self
            .db
            .query(format!(
                "SELECT * FROM type::table($table) WHERE {column} = type::string($value)"
            ))
            .bind(("table", ACCOUNT))
            .bind(("value", column.value()))
            .await?
            .take(0)?;

        Ok(account.map(Into::into))
    }
}

#[cfg(test)]
pub mod mock {
    use tokio::sync::Mutex;

    use super::*;

    pub struct AccountRepositoryImpl {
        pub accounts: Mutex<Vec<Account>>,
    }

    #[async_trait]
    impl AccountRepository for AccountRepositoryImpl {
        async fn is_account(&self, email: &str) -> RepositoryResult<bool> {
            let accounts = self.accounts.lock().await;
            Ok(accounts.iter().any(|a| a.email == email))
        }

        async fn signup(&self, account: CreateAccount) -> RepositoryResult<Account> {
            let mut accounts = self.accounts.lock().await;

            let acc = Account {
                id: "ajkf".to_string(),
                name: account.name.to_owned(),
                email: account.email.to_owned(),
                password: account.password.to_owned(),
            };

            accounts.push(acc.clone());

            Ok(acc)
        }

        async fn find_one(&self, column: FindByCol) -> RepositoryResult<Option<Account>> {
            let accounts = self.accounts.lock().await;

            match column {
                FindByCol::Email(email) => {
                    let account = accounts.iter().find(|a| a.email == email).cloned();
                    Ok(account)
                }
            }
        }
    }
}
