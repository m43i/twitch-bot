use crate::entity::bot as bot_entity;
use anyhow::{Error, Result};
use sea_orm::prelude::*;

/**
 * Get a bot refresh token
 */
pub async fn get_bot_refresh_token<T: ConnectionTrait>(bot: &str, db: &T) -> Result<String, Error> {
    let bot = bot_entity::Entity::find_by_id(bot).one(db).await?;

    let bot = match bot {
        Some(bot) => bot,
        None => {
            println!("No bot found");
            return Err(Error::msg("No bot found"));
        }
    };

    return Ok(bot.refresh_token);
}

/**
 * Update a bot refresh token with a new refresh token value
 */
pub async fn update_bot_refresh_token<T: ConnectionTrait>(
    db: &T,
    refresh_token: &str,
) -> Result<(), Error> {
    let bot = bot_entity::Entity::find_by_id("dustin").one(db).await?;

    let bot = match bot {
        Some(bot) => bot,
        None => {
            println!("No bot found");
            return Err(Error::msg("No bot found"));
        }
    };

    let mut bot: bot_entity::ActiveModel = bot.into();
    bot.refresh_token = sea_orm::ActiveValue::Set(String::from(refresh_token));
    bot.update(db).await?;

    return Ok(());
}
