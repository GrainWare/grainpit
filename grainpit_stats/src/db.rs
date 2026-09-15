use chrono::DateTime;
use grainpit::stats::Submission;
use ipnet::IpNet;
use sqlx::{PgPool, QueryBuilder, Row, prelude::FromRow};
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

pub async fn get_grainpit_urls(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query("SELECT unnest(grainpit_urls) FROM account;")
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.get(0)).collect())
}

pub async fn insert_submission(
    pool: &PgPool,
    submission: Submission,
    user_id: i32,
) -> Result<(), sqlx::Error> {
    let mut query_builder =
        QueryBuilder::new("INSERT INTO request (time, creator, url, ip, user_agent) ");

    query_builder.push_values(submission.requests, |mut b, request| {
        b.push_bind(request.time)
            .push_bind(user_id)
            .push_bind(request.url)
            .push_bind(request.ip)
            .push_bind(request.user_agent);
    });

    let query = query_builder.build();

    query.execute(pool).await?;
    Ok(())
}

pub struct Stats {
    pub unique_ips: u32,
    pub unique_uas: u32,
    pub total_requests: u64,
    pub grainpit_url_count: u32,
}

pub async fn get_stats(pool: &PgPool) -> Result<Stats, sqlx::Error> {
    let row = sqlx::query(
        r#"SELECT
            COUNT(DISTINCT ip) AS unique_ips,
            COUNT(DISTINCT user_agent) AS unique_uas,
            COUNT(*) AS total_requests,
            (
                SELECT COUNT(*)
                FROM (
                    SELECT unnest(grainpit_urls) AS url
                    FROM account
                ) AS all_urls
            ) AS grainpit_url_count
        FROM request"#,
    )
    .fetch_one(pool)
    .await?;

    Ok(Stats {
        unique_ips: row.try_get::<i64, _>("unique_ips")? as u32,
        unique_uas: row.try_get::<i64, _>("unique_uas")? as u32,
        total_requests: row.try_get::<i64, _>("total_requests")? as u64,
        grainpit_url_count: row.try_get::<i64, _>("grainpit_url_count")? as u32,
    })
}

pub async fn create_user(pool: &PgPool, name: String) -> Result<Uuid, sqlx::Error> {
    let key = Uuid::new_v4();
    sqlx::query("INSERT INTO account (key, name) VALUES ($1, $2)")
        .bind(key)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(key)
}
