use anyhow::{Error, Result};
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

/**
 * Connect to the twitch IRC server
 */
pub async fn connect(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    let (ws, _) = connect_async(url).await?;
    Ok(ws)
}

/**
 * Send a message to the server
 */
pub async fn send_message(
    msg: &str,
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), Error> {
    ws.send(tokio_tungstenite::tungstenite::Message::Text(
        msg.to_string(),
    ))
    .await?;
    return Ok(());
}

/**
 * Authenticate with the server
 */
pub async fn auth_message(
    token: &str,
    nick: &str,
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), Error> {
    send_message(
        "CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership",
        ws,
    )
    .await?;
    send_message(&format!("PASS oauth:{}", token), ws).await?;
    send_message(&format!("NICK {}", nick), ws).await?;

    return Ok(());
}

/**
 * Join a channel
 */
pub async fn join_channels(
    channel: Vec<&str>,
    write: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), Error> {
    let join_string = channel
        .iter()
        .map(|x| format!("#{}", x))
        .collect::<Vec<String>>()
        .join(",");

    write
        .send(Message::Text(format!("JOIN {}", join_string)))
        .await?;

    return Ok(());
}
