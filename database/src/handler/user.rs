use crate::entity::user as user_entity;
use crate::sea_orm::prelude::*;
use anyhow::{Error, Result};
use sea_orm::ActiveValue::NotSet;

/**
 * Create a new user
 */
pub async fn create_user(
    user: user_entity::ActiveModel,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    let insert = user_entity::Entity::insert(user)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(user_entity::Column::Id)
                .update_column(user_entity::Column::Id)
                .to_owned(),
        )
        .exec(db)
        .await;

    return match insert {
        Ok(_) => Ok(()),
        Err(e) => {
            match e {
                sea_orm::error::DbErr::RecordNotInserted => Ok(()),
                _ => Err(Error::new(e)),
            }
        }
    }
}

/**
 * Create many users
 */
pub async fn create_many(
    users: Vec<user_entity::ActiveModel>,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    let insert = user_entity::Entity::insert_many(users)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(user_entity::Column::Id)
                .update_column(user_entity::Column::Id)
                .to_owned(),
        )
        .exec(db)
        .await;

    return match insert {
        Ok(_) => Ok(()),
        Err(e) => {
            match e {
                sea_orm::error::DbErr::RecordNotInserted => Ok(()),
                _ => Err(Error::new(e)),
            }
        }
    }
}

/**
 * Get a user by id
 */
pub async fn get_user(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Option<user_entity::Model>, Error> {
    let user = user_entity::Entity::find_by_id(user_id).one(db).await?;
    return Ok(user);
}

/**
 * Delete a user
 */
pub async fn delete_user(user_id: i32, db: &DatabaseConnection) -> Result<(), Error> {
    let user = user_entity::Entity::find_by_id(user_id).one(db).await?;
    if user.is_some() {
        let user = user.unwrap();
        user.delete(db).await?;
    }
    return Ok(());
}

/**
 * Deactivate a user by deleting private information
 */
pub async fn deactivate_user(user_id: i32, db: &DatabaseConnection) -> Result<(), Error> {
    let user = user_entity::Entity::find_by_id(user_id).one(db).await?;
    if user.is_some() {
        let user = user.unwrap();
        let mut user: user_entity::ActiveModel = user.into();
        user.email = NotSet;
        user.update(db).await?;
    }
    return Ok(());
}
