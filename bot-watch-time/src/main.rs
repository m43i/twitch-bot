use anyhow::{Error, Result};
use database::{entity::channel as channel_entity, sea_orm::{ActiveValue, TransactionTrait}};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let client_secret =
        std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_SECRET not set");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");

    let db = database::connect(&db_url).await?;
    let mut redis = cache::connect(&redis_url).await?;

    loop {
        let bot = database::handler::bot::get_bot("dustin", &db).await?;
        let token =
            auth::token::get_bot_token("dustin", &client_id, &client_secret, &db, &mut redis)
                .await?;

        let channels: Vec<channel_entity::Model> =
            database::handler::channel::get_live_channels(&db).await?;

        for channel in channels {
            let mut chatters: Vec<twitch_api::Chatter> = Vec::new();
            match twitch_api::get_chatters(
                &client_id,
                &token,
                bot.twitch_id,
                channel.id,
                "",
                &mut chatters,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };

            let mut users: Vec<database::entity::user::ActiveModel> = Vec::new();
            for chatter in chatters.clone() {
                let user_id = i32::from_str_radix(&chatter.user_id, 10).unwrap();
                let user = database::entity::user::ActiveModel {
                    id: ActiveValue::Set(user_id),
                    nick: ActiveValue::Set(chatter.user_login),
                    display_name: ActiveValue::Set(chatter.user_name),
                    ..Default::default()
                };

                users.push(user);
            }

            let txn = match db.begin().await {
                Ok(txn) => txn,
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };

            match database::handler::user::create_many(users, &txn).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };

            let currently_watching =
                database::handler::watchtime::get_currently_watching(channel.id, &txn).await;
            let currently_watching = match currently_watching {
                Ok(watching) => watching,
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };

            let no_longer_watching = currently_watching
                .iter()
                .filter(|user| {
                    !chatters.iter().any(|chatter| {
                        i32::from_str_radix(&chatter.user_id, 10).unwrap() == user.user_id
                    })
                })
                .map(|user| user.user_id.clone())
                .collect::<Vec<i32>>();
            let new_watching = chatters
                .iter()
                .map(|chatter| i32::from_str_radix(&chatter.user_id, 10).unwrap())
                .filter(|chatter| {
                    !currently_watching
                        .iter()
                        .any(|user| chatter == &user.user_id)
                })
                .collect::<Vec<i32>>();

            if no_longer_watching.len() > 0 {
                match database::handler::watchtime::stop_watching(
                    channel.id,
                    no_longer_watching,
                    &txn,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                };
            }
            if new_watching.len() > 0 {
                match database::handler::watchtime::start_watching(channel.id, new_watching, &txn)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                };
            }

            match txn.commit().await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
        }

        tokio::time::sleep(std::time::Duration::from_secs(2 * 60)).await;
    }
}
