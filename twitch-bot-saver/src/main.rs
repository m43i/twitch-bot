mod handler;

use std::sync::Arc;

use anyhow::{Error, Result};
use dotenvy::dotenv;
use tokio::sync::Mutex;
use utils::database::entity::user as user_entity;
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

    let chat_messages: Arc<Mutex<Vec<chat_message_entity::ActiveModel>>> = Arc::new(Mutex::new(vec![]));
    let users: Arc<Mutex<Vec<user_entity::ActiveModel>>> = Arc::new(Mutex::new(vec![]));

    let chat_messages_clone = chat_messages.clone();
    let users_clone = users.clone();

    let save_loop = tokio::spawn(async move {
        let db = utils::database::db::connect(&db_url).await;
        let db = match db {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to connect to database: {:?}", e);
                return;
            },
        };
        loop {
            let mut messages = chat_messages_clone.lock().await;
            let mut users = users_clone.lock().await;

            if messages.len() > 0 {
                let save = save_chat_messages(&db, messages.clone(), users.clone()).await;
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
            let parsed_message = match utils::parser::irc_parser::parse(&message.to_string()).await
            {
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
                    match handler::handle_privmsg_save(&parsed_message, &mut chat_messages, &mut users).await {
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
                    match handler::handle_clearmsg_update(&parsed_message, &mut chat_messages, &db).await {
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

/**
 * Save chat messages to database
 */
async fn save_chat_messages(
    db: &DatabaseConnection,
    chat_messages: Vec<chat_message_entity::ActiveModel>,
    users: Vec<user_entity::ActiveModel>,
) -> Result<(), Error> {

    if users.len() > 0 {
        match user_entity::Entity::insert_many(users)
            .on_conflict(
                utils::sea_orm::sea_query::OnConflict::column(user_entity::Column::Id)
                .update_column(user_entity::Column::Nick)
                .update_column(user_entity::Column::DisplayName)
                .update_column(user_entity::Column::UpdatedAt)
                .to_owned()
            )
            .exec(db)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to insert viewers: {:?}", e);
            },
        };
    }

    return match chat_message_entity::Entity::insert_many(chat_messages)
        .exec(db)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(Error::from(e)),
    };
}
