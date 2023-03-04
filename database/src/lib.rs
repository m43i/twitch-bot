pub mod entity;
pub mod handler;

use sea_orm::{DatabaseConnection, Database};
use anyhow::{Result, Error};

pub extern crate sea_orm;

pub async fn connect(url: &str) -> Result<DatabaseConnection, Error> {
    let db: DatabaseConnection = Database::connect(url).await?;

    return Ok(db)
}
