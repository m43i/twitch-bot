use database::sea_orm::DatabaseConnection;
use anyhow::{Result, Error};

/**
 * Delete a user
 */
pub async fn delete_user(user_id: i32, db: &DatabaseConnection) -> Result<(), Error> {
    database::handler::user::delete_user(user_id, db).await?;
    return Ok(());
}

/**
 * Deactivate a user also deactivates the channel
 */
pub async fn deactivate_user(user_id: i32, db: &DatabaseConnection) -> Result<(), Error> {
    database::handler::user::deactivate_user(user_id, db).await?;
    database::handler::channel::deactivate_channel(user_id, db).await?;
    return Ok(());
}
