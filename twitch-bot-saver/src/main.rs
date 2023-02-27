mod handler;

use anyhow::{Error, Result};
use dotenvy::dotenv;
use utils::database::entity::chat_message as chat_message_entity;
use utils::futures_util::StreamExt;
use utils::parser::irc_parser::IRCCommandType;
use utils::sea_orm::{DatabaseConnection, EntityTrait};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token = std::env::var("TWITCH_TOKEN").expect("TWITCH_TOKEN must be set");
    let nick = std::env::var("TWITCH_NICK").expect("TWITCH_NICK must be set");
    let endpoint = std::env::var("TWITCH_WS_ENDPOINT").expect("TWITCH_ENDPOINT must be set");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let ws = utils::websocket::client::connect(&endpoint).await;
    let db = utils::database::db::connect(&db_url).await;

    let db = match db {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to database")),
    };

    let mut ws = match ws {
        Ok(x) => x,
        Err(_) => return Err(Error::msg("Failed to connect to websocket")),
    };

    let auth = utils::websocket::client::auth_message(&token, &nick, &mut ws).await;

    match auth {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to authenticate")),
    }

    let join =
        utils::websocket::client::join_channels(vec!["striikzx", "anniislost"], &mut ws).await;

    match join {
        Ok(_) => (),
        Err(_) => return Err(Error::msg("Failed to join channels")),
    }

    let mut chat_messages = vec![];
    let mut last_insert = chrono::Utc::now();

    while let Some(msg) = ws.next().await {
        if chat_messages.len() >= 100
            || (chat_messages.len() > 0
                && chrono::Utc::now() - last_insert > chrono::Duration::minutes(15))
        {
            match save_chat_messages(&db, chat_messages.clone()).await {
                Ok(_) => {
                    chat_messages = vec![];
                    last_insert = chrono::Utc::now();
                },
                Err(e) => {
                    println!("Failed to insert chat messages: {:?}", e);
                },
            };
        }

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
            let parsed_message =
                match utils::parser::irc_parser::parse(&message.to_string()).await {
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error parsing message: {}", message);
                        println!("Error: {:?}", e);
                        continue;
                    }
                };

            let handle = match parsed_message.command.command {
                IRCCommandType::PING => handler::handle_ping(&mut ws).await,
                IRCCommandType::PRIVMSG => {
                    handler::handle_privmsg_save(&parsed_message, &mut chat_messages)
                }
                IRCCommandType::CLEARMSG => handler::handle_clearmsg_update(&parsed_message, &mut chat_messages, &db).await,
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

    if chat_messages.len() > 0 {
        match save_chat_messages(&db, chat_messages).await {
            Ok(_) => (),
            Err(_) => return Err(Error::msg("Failed to insert chat messages")),
        }
    }

    return Ok(());
}

async fn save_chat_messages(
    db: &DatabaseConnection,
    chat_messages: Vec<chat_message_entity::ActiveModel>,
) -> Result<(), Error> {
    return match chat_message_entity::Entity::insert_many(chat_messages)
        .exec(db)
        .await {
        Ok(_) => Ok(()),
        Err(e) => return Err(Error::from(e)),
    };
}
