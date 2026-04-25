#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;
#[cfg(feature = "server")]
use serde::de::DeserializeOwned;
#[cfg(feature = "server")]
use sqlx::Row;

#[cfg(feature = "server")]
fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set. Create a .env file or set the environment variable.")
}

#[cfg(feature = "server")]
static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

#[cfg(feature = "server")]
async fn db_pool() -> Result<&'static sqlx::PgPool, String> {
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool);
    }

    let url = database_url();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(|e| format!("DB connection failed: {e}"))?;
    let _ = DB_POOL.set(pool);
    Ok(DB_POOL.get().unwrap())
}

#[cfg(feature = "server")]
pub(crate) fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(feature = "server")]
pub(crate) fn server_error(message: String) -> ServerFnError {
    ServerFnError::new(message)
}

#[cfg(feature = "server")]
pub(crate) async fn run_json_query<T>(sql: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let pool = db_pool().await?;
    let row = sqlx::query(sql)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("query failed: {e}"))?;
    let payload: serde_json::Value = row.get(0);
    serde_json::from_value(payload).map_err(|e| format!("failed to parse postgres json: {e}"))
}

#[cfg(feature = "server")]
pub(crate) async fn run_scalar_i64(sql: &str) -> Result<i64, String> {
    let pool = db_pool().await?;
    let row = sqlx::query(sql)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("query failed: {e}"))?;
    let val: i64 = match row.try_get::<i32, _>(0) {
        Ok(v) => v as i64,
        Err(_) => row
            .try_get::<i64, _>(0)
            .map_err(|e| format!("scalar decode failed: {e}"))?,
    };
    Ok(val)
}

#[cfg(feature = "server")]
pub(crate) async fn run_exec(sql: &str) -> Result<(), String> {
    let pool = db_pool().await?;
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| format!("exec failed: {e}"))?;
    Ok(())
}
