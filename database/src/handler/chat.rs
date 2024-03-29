use sea_orm::prelude::*;
use sea_orm::TransactionTrait;
use crate::entity::chat_message as chat_message_entity;
use crate::entity::user as user_entity;
use anyhow::{Result, Error};

/**
 * Save chat messages to database
 */
pub async fn save_chat_messages(
    db: &DatabaseConnection,
    chat_messages: Vec<chat_message_entity::ActiveModel>,
    users: Vec<user_entity::ActiveModel>,
) -> Result<(), Error> {
    let txn = db.begin().await?;

    if users.len() > 0 {
        crate::handler::user::create_many(users, &txn).await?;
    }
    chat_message_entity::Entity::insert_many(chat_messages)
        .exec(&txn)
        .await?;

    txn.commit().await?;
    
    return Ok(());
}
