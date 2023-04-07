use anyhow::{Result, Error};
use sea_orm::prelude::*;
use crate::entity::bot as bot_entity;

pub async fn get_bot<T: ConnectionTrait>(bot: &str, db: &T) -> Result<bot_entity::Model, Error> {
    let bot_model = bot_entity::Entity::find_by_id(bot).one(db).await?;
    let bot_model = match bot_model {
        Some(x) => x,
        None => return Err(Error::msg("Bot not found")),
    };

    return Ok(bot_model);
}
