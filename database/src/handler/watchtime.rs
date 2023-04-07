use crate::entity::watch_time as watch_time_entity;
use anyhow::{Error, Result};
use sea_orm::{prelude::*, sea_query::Expr, Set};

/**
 * Get all users currently watching a channel
 */
pub async fn get_currently_watching<T: ConnectionTrait>(
    channel_id: i32,
    db: &T,
) -> Result<Vec<watch_time_entity::Model>, Error> {
    let watch_time: Vec<watch_time_entity::Model> = watch_time_entity::Entity::find()
        .filter(watch_time_entity::Column::BoardcasterId.eq(channel_id))
        .filter(watch_time_entity::Column::EndedAt.is_null())
        .all(db)
        .await?;

    return Ok(watch_time);
}

/**
 * Start watching a channel for a list of users
 */
pub async fn start_watching<T: ConnectionTrait>(
    channel_id: i32,
    user_id: Vec<i32>,
    db: &T,
) -> Result<(), Error> {
    let mut watch_time: Vec<watch_time_entity::ActiveModel> = Vec::new();

    for user in user_id {
        let watch_time_entity = watch_time_entity::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            boardcaster_id: Set(channel_id),
            user_id: Set(user),
            started_at: Set(chrono::Utc::now()),
            ended_at: Set(None),
            ..Default::default()
        };

        watch_time.push(watch_time_entity);
    }

    watch_time_entity::Entity::insert_many(watch_time)
        .exec(db)
        .await?;

    return Ok(());
}

/**
 * Stop watching for a list of users in a given channel
 */
pub async fn stop_watching<T: ConnectionTrait>(
    channel_id: i32,
    user_id: Vec<i32>,
    db: &T,
) -> Result<(), Error> {
    watch_time_entity::Entity::update_many()
        .col_expr(
            watch_time_entity::Column::EndedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(watch_time_entity::Column::UserId.is_in(user_id))
        .filter(watch_time_entity::Column::BoardcasterId.eq(channel_id))
        .exec(db)
        .await?;

    return Ok(());
}
