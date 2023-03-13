use crate::entity::channel as channel_entity;
use anyhow::{Error, Result};
use sea_orm::{prelude::*, ActiveValue};

/**
 * Get a channel by id
 */
pub async fn get_channel(
    channel: i32,
    db: &DatabaseConnection,
) -> Result<Option<channel_entity::Model>, Error> {
    let channel = channel_entity::Entity::find_by_id(channel).one(db).await?;
    return Ok(channel);
}

/**
 * Deactive a channel
 */
pub async fn deactivate_channel(channel: i32, db: &DatabaseConnection) -> Result<(), Error> {
    let channel = channel_entity::Entity::find_by_id(channel).one(db).await?;
    let channel = match channel {
        Some(channel) => channel,
        None => return Ok(()),
    };

    let mut channel: channel_entity::ActiveModel = channel.into();
    channel.active = ActiveValue::Set(false as i8);
    channel.live = ActiveValue::Set(false as i8);
    channel.update(db).await?;

    return Ok(());
}
