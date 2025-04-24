use std::sync::Arc;

use crate::domain::{
    error::{AppError, AppResult},
    models::account::{Account, CreateAccount, Credentials},
    repositories::account::{AccountRepository, FindByCol},
    services::account::AccountService,
};

use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, Result, SaltString, rand_core::OsRng,
    },
};

use async_trait::async_trait;

pub struct AccountServiceImpl {
    repository: Arc<dyn AccountRepository>,
}

impl AccountServiceImpl {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    async fn is_account(&self, email: &str) -> AppResult<bool> {
        Ok(self.repository.is_account(email).await?)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<Account>> {
        Ok(self
            .repository
            .find_one(FindByCol::Email(email.to_string()))
            .await?)
    }
}

#[async_trait]
impl AccountService for AccountServiceImpl {
    async fn signup(&self, mut new_account: CreateAccount) -> AppResult<Account> {
        if self.is_account(&new_account.email).await? {
            return Err(AppError::Conflict("Account already exists"));
        }

        new_account.password = encrypt_password(&new_account.password)?;

        Ok(self.repository.signup(new_account).await?)
    }

    async fn signin(&self, credentials: Credentials) -> AppResult<Account> {
        let account = match self.find_by_email(&credentials.email).await? {
            Some(account) => account,
            None => return Err(AppError::Unauthorized()),
        };

        verify_password(&credentials.password, &account.password)?;

        Ok(account)
    }
}

pub fn encrypt_password(password: &str) -> Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<()> {
    let argon2 = Argon2::default();
    let hash = PasswordHash::new(hash);

    argon2.verify_password(password.as_bytes(), &hash?)
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use super::*;
    use crate::infrastructure::repositories::account::mock::AccountRepositoryImpl;
    use rstest::*;

    #[fixture]
    fn service() -> AccountServiceImpl {
        let repo = Arc::new(AccountRepositoryImpl {
            accounts: Mutex::new(
                [Account {
                    id: "1".to_string(),
                    name: "Test".to_string(),
                    email: "test_account@spacecraft.com".to_string(),
                    password: encrypt_password("p4ssw0rd").unwrap(),
                }]
                .to_vec(),
            ),
        });
        AccountServiceImpl::new(repo.clone())
    }

    #[rstest]
    #[tokio::test]
    async fn test_signup_success(service: AccountServiceImpl) {
        let result = service
            .signup(CreateAccount {
                name: "Test".to_string(),
                email: "new_account@spacecraft.com".to_string(),
                password: "p4ssw0rd".to_string(),
            })
            .await;

        let account = result.unwrap();

        assert_eq!(account.email, "new_account@spacecraft.com");
        assert!(verify_password("p4ssw0rd", &account.password).is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_signup_conflict(service: AccountServiceImpl) {
        let result = service
            .signup(CreateAccount {
                name: "Test".to_string(),
                email: "test_account@spacecraft.com".to_string(),
                password: "p4ssw0rd".to_string(),
            })
            .await;

        assert_eq!(
            result.unwrap_err(),
            AppError::Conflict("Account already exists")
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_signin_success(service: AccountServiceImpl) {
        let result = service
            .signin(Credentials {
                email: "test_account@spacecraft.com".to_string(),
                password: "p4ssw0rd".to_string(),
            })
            .await;

        assert_eq!(result.unwrap().email, "test_account@spacecraft.com");
    }

    #[rstest]
    #[tokio::test]
    async fn test_signin_wrong_password(service: AccountServiceImpl) {
        let result = service
            .signin(Credentials {
                email: "test_account@spacecraft.com".to_string(),
                password: "wrongpassword".to_string(),
            })
            .await;

        assert_eq!(result.unwrap_err(), AppError::Unauthorized());
    }
}
