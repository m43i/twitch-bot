use anyhow::{Error, Result};
use auth::token::get_bot_token;
use cache::redis::aio::Connection;
use database::sea_orm::DatabaseConnection;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{client::connect, messages::auth_message};

pub struct Config {
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub nick: String,
}

/**
 * Get config from environment variables
 * Setup database connection and websocket
 * Authenticate with twitch
 */
pub async fn websocket_config(
    bot: &str,
    endpoint: &str,
    client_id: &str,
    client_secret: &str,
    db: &DatabaseConnection,
    redis: &mut Connection,
) -> Result<Config, Error> {
    let ws = connect(&endpoint).await;

    let mut ws = match ws {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to websocket")),
    };

    let token = get_bot_token(bot, client_id, client_secret, db, redis).await?;
    let bot_model = database::handler::bot::get_bot(bot, db).await?;
    let nick = bot_model.nick;

    let auth = auth_message(&token, &nick, &mut ws).await;

    match auth {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to authenticate")),
    }

    return Ok(Config { ws, nick });
}
