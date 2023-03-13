mod handler;

use anyhow::{Error, Result};
use database::entity::chat_message as Chat_Message;
use database::entity::user as User;
use dotenvy::dotenv;
use parser::irc_parser::IRCCommandType;
use std::sync::Arc;
use tokio::sync::Mutex;
use websocket::client::connect_channels;
use websocket::config::websocket_config;
use websocket::futures_util::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let ws_endpoint = std::env::var("TWITCH_WS_ENDPOINT").expect("WS_ENDPOINT not set");
    let db_endpoint = std::env::var("DATABASE_URL").expect("DB_URL not set");
    let redis_endpoint = std::env::var("REDIS_URL").expect("REDIS_URL not set");
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("CLIENT_ID not set");
    let client_secret = std::env::var("TWITCH_CLIENT_SECRET").expect("CLIENT_SECRET not set");

    let db = database::connect(&db_endpoint).await;
    let db = match db {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to database")),
    };

    let redis = cache::connect(&redis_endpoint).await;
    let mut redis = match redis {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to redis")),
    };

    let config = websocket_config(
        "dustin",
        &ws_endpoint,
        &client_id,
        &client_secret,
        &db,
        &mut redis,
    )
    .await;
    let config = match config {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to get config")),
    };

    let mut ws = config.ws;

    match connect_channels(&db, &mut ws).await {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to join channels")),
    }

    let chat_messages: Arc<Mutex<Vec<Chat_Message::ActiveModel>>> = Arc::new(Mutex::new(vec![]));
    let users: Arc<Mutex<Vec<User::ActiveModel>>> = Arc::new(Mutex::new(vec![]));

    let chat_messages_clone = chat_messages.clone();
    let users_clone = users.clone();
    let db_clone = db.clone();

    let save_loop = tokio::spawn(async move {
        loop {
            let mut messages = chat_messages_clone.lock().await;
            let mut users = users_clone.lock().await;

            if messages.len() > 0 {
                let save = database::handler::chat::save_chat_messages(
                    &db_clone,
                    messages.clone(),
                    users.clone(),
                )
                .await;
                match save {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Failed to save chat messages: {:?}", e);
                    }
                }
            }

            messages.clear();
            users.clear();

            drop(messages);
            drop(users);

            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    while let Some(msg) = ws.next().await {
        let msg = match msg {
            Ok(x) => x,
            Err(_) => continue,
        };

        let messages = match msg.into_text() {
            Ok(x) => x,
            Err(_) => continue,
        };

        let messages = messages.split("\r\n").collect::<Vec<&str>>();

        for message in messages {
            let parsed_message = match parser::irc_parser::parse(&message.to_string()).await {
                Ok(x) => x,
                Err(e) => {
                    println!("Error parsing message: {}", message);
                    println!("Error: {:?}", e);
                    continue;
                }
            };

            let handle = match parsed_message.command.command {
                IRCCommandType::PING => handler::handle_ping(&mut ws).await,
                IRCCommandType::PRIVMSG => Ok({
                    let mut chat_messages = chat_messages.lock().await;
                    let mut users = users.lock().await;
                    match handler::handle_privmsg_save(
                        &parsed_message,
                        &mut chat_messages,
                        &mut users,
                    )
                    .await
                    {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error handling privmsg: {}", message);
                            println!("Error: {:?}", e);
                        }
                    };
                    drop(chat_messages);
                    drop(users);
                }),
                IRCCommandType::CLEARMSG => Ok({
                    let mut chat_messages = chat_messages.lock().await;
                    match handler::handle_clearmsg_update(&parsed_message, &mut chat_messages, &db)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error handling clearmsg: {}", message);
                            println!("Error: {:?}", e);
                        }
                    };
                    drop(chat_messages);
                }),
                _ => continue,
            };

            match handle {
                Ok(_) => continue,
                Err(e) => {
                    println!("Error handling message: {}", message);
                    println!("Error: {:?}", e);
                    continue;
                }
            }
        }
    }

    match save_loop.await {
        Ok(_) => (),
        Err(e) => {
            println!("Error in save loop: {:?}", e);
        }
    };

    return Ok(());
}
