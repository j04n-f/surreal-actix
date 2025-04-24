use std::sync::Arc;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

use crate::domain::repositories::account::AccountRepository;
use crate::domain::services::account::AccountService;
use crate::domain::services::jsonwebtoken::JsonWebTokenService;

use crate::services::account::AccountServiceImpl;
use crate::services::jsonwebtoken::{JsonWebTokenServiceImpl, KeyPair};

use crate::infrastructure::repositories::account::AccountRepositoryImpl;

pub struct Container {
    pub account_service: Arc<dyn AccountService>,
    pub jsonwebtoken_service: Arc<dyn JsonWebTokenService>,
}

impl Container {
    pub fn new(conn: Surreal<Client>, keys: KeyPair) -> Self {
        let db = Arc::new(conn);

        Container {
            account_service: account_service(db.clone()),
            jsonwebtoken_service: jsonwebtoken_service(keys),
        }
    }
}

fn account_service(db: Arc<Surreal<Client>>) -> Arc<dyn AccountService> {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(AccountRepositoryImpl::new(db.clone()));

    Arc::new(AccountServiceImpl::new(account_repository))
}

fn jsonwebtoken_service(keys: KeyPair) -> Arc<dyn JsonWebTokenService> {
    Arc::new(JsonWebTokenServiceImpl::new(keys))
}
