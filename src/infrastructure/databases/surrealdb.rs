use crate::config::SurrealDbConfig;

use surrealdb::{
    Error, Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};

pub async fn connect(db_config: &SurrealDbConfig) -> Result<Surreal<Client>, Error> {
    let db = Surreal::new::<Ws>(format!("{}:{}", db_config.host, db_config.port)).await?;

    db.signin(Root {
        username: db_config.username.as_str(),
        password: db_config.password.as_str(),
    })
    .await?;

    db.use_ns(db_config.namespace.as_str())
        .use_db(db_config.database.as_str())
        .await?;

    Ok(db)
}
