use crate::domain::models::account::Account;
use ::surrealdb::{Surreal, engine::remote::ws::Client, sql::Thing};
use chrono;

pub async fn seed_account(conn: &Surreal<Client>) -> Account {
    let query = format!(
        r#"
        LET $account = (CREATE account CONTENT {{
            name: '{}',
            email: '{}',
            password: crypto::argon2::generate('{}')
        }});
        RETURN $account[0].id;
        "#,
        "Test Account", "test_account@email.com", "stR0ngP4ssw0rd!"
    );

    let thing: Option<Thing> = conn.query(query).await.unwrap().take(1).unwrap();

    Account {
        id: thing.unwrap().id.to_string(),
        name: "Test Account".to_string(),
        email: "test_account@email.com".to_string(),
        password: "stR0ngP4ssw0rd!".to_string(),
    }
}
