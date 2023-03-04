use anyhow::{Error, Result};
use database::entity::channel as channel_entity;
use database::entity::user as user_entity;
use database::sea_orm::prelude::*;
use database::sea_orm::DatabaseConnection;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::messages::join_channels_message;

/**
 * Connect to the twitch IRC server
 */
pub async fn connect(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    let (ws, _) = connect_async(url).await?;
    Ok(ws)
}

pub async fn connect_channels(
    db: &DatabaseConnection,
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), Error> {
    let channels: Vec<channel_entity::Model> = match channel_entity::Entity::find()
        .filter(channel_entity::Column::Active.eq(1))
        .all(db)
        .await
    {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to get channels")),
    };

    let users = match channels.load_one(user_entity::Entity, db).await {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to get users")),
    };

    let users_vec: Vec<String> = users
        .into_iter()
        .filter_map(|x| {
            if x.is_none() {
                return None;
            }

            let user = x.unwrap();
            let nick = user.nick;

            return Some(nick);
        })
        .collect::<Vec<String>>();

    let join = join_channels_message(users_vec.iter().map(|x| x.as_str()).collect(), ws).await;

    match join {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to join channels")),
    }
    return Ok(());
}
