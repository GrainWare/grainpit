use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Mutex;
use tracing::warn;

#[derive(Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub ip_failed_auths: Mutex<HashMap<IpAddr, u32>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_options =
            PgConnectOptions::from_str("postgresql://postgres:example@localhost:5432/postgres")?
                .disable_statement_logging()
                .to_owned();

        let pool = PgPoolOptions::new().connect_with(db_options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        if !sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM account WHERE name = $1)")
            .bind("admin")
            .fetch_one(&pool)
            .await?
        {
            let key = uuid::Uuid::new_v4();
            sqlx::query("INSERT INTO account (name, key) VALUES ($1, $2);")
                .bind("admin")
                .bind(key)
                .execute(&pool)
                .await
                .unwrap();
            warn!(
                "created admin account with key {} (this will not be printed again, keep the key safe)",
                key
            )
        }

        Ok(Self {
            pool,
            ip_failed_auths: Mutex::new(HashMap::new()),
        })
    }
}
