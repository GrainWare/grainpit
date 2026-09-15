use chrono::DateTime;
use ipnet::IpNet;
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub key: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub grainpit_urls: Vec<String>,
}

#[derive(FromRow, Debug)]
pub struct Request {
    pub time: DateTime<chrono::Utc>,
    pub creator: i32,
    pub url: String,
    pub ip: IpNet,
    pub user_agent: String,
}

pub async fn get_account_from_key(pool: &PgPool, key: Uuid) -> Result<Account, sqlx::Error> {
    sqlx::query_as("SELECT * FROM account WHERE key = $1")
        .bind(key)
        .fetch_one(pool)
        .await
}

pub async fn key_valid(pool: &PgPool, key: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM account WHERE key = $1)")
        .bind(key)
        .fetch_one(pool)
        .await
}

pub async fn edit_user_grainpit_urls(
    pool: &PgPool,
    key: Uuid,
    urls: Vec<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE account SET grainpit_urls = $1 WHERE key = $2")
        .bind(urls)
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}
