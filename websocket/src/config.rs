use anyhow::{Error, Result};
use database::sea_orm::DatabaseConnection;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{client::connect, messages::auth_message};

pub struct Config {
    pub db: DatabaseConnection,
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

/**
 * Get config from environment variables
 * Setup database connection and websocket
 * Authenticate with twitch
 */
pub async fn websocket_config() -> Result<Config, Error> {
    let token = std::env::var("TWITCH_TOKEN").expect("TWITCH_TOKEN must be set");
    let nick = std::env::var("TWITCH_NICK").expect("TWITCH_NICK must be set");
    let endpoint = std::env::var("TWITCH_WS_ENDPOINT").expect("TWITCH_ENDPOINT must be set");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let ws = connect(&endpoint).await;
    let db = database::connect(&db_url).await;

    let db = match db {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to database")),
    };

    let mut ws = match ws {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to websocket")),
    };

    let auth = auth_message(&token, &nick, &mut ws).await;

    match auth {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to authenticate")),
    }

    return Ok(Config { db, ws });
}
