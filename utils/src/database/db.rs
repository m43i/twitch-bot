use anyhow::{Result, Error};
use sea_orm::{DatabaseConnection, Database};

pub async fn connect(url: &str) -> Result<DatabaseConnection, Error> {
    let db: DatabaseConnection = Database::connect(url).await?;

    return Ok(db)
}
