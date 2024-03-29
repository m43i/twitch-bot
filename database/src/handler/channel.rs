use crate::entity::channel as channel_entity;
use anyhow::{Error, Result};
use sea_orm::{prelude::*, ActiveValue};

/**
 * Get a channel by id
 */
pub async fn get_channel<T: ConnectionTrait>(
    channel: i32,
    db: &T,
) -> Result<Option<channel_entity::Model>, Error> {
    let channel = channel_entity::Entity::find_by_id(channel).one(db).await?;
    return Ok(channel);
}

/**
 * Get live channels
 */
pub async fn get_live_channels<T: ConnectionTrait>(
    db: &T,
) -> Result<Vec<channel_entity::Model>, Error> {
    let channels = channel_entity::Entity::find()
        .filter(channel_entity::Column::Live.eq(true as i8))
        .filter(channel_entity::Column::Active.eq(true as i8))
        .all(db)
        .await?;
    return Ok(channels);
}

/**
 * Deactive a channel
 */
pub async fn deactivate_channel<T: ConnectionTrait>(channel: i32, db: &T) -> Result<(), Error> {
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
