use sea_orm::DatabaseConnection;
use sea_orm::prelude::*;
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
    if users.len() > 0 {
        match user_entity::Entity::insert_many(users)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(user_entity::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                if e != sea_orm::error::DbErr::RecordNotInserted {
                    println!("Failed to insert viewers: {:?}", e);
                }
            }
        };
    }

    return match chat_message_entity::Entity::insert_many(chat_messages)
        .exec(db)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(Error::from(e)),
    };
}
