use anyhow::{Error, Result};
use serde::Deserialize;
use async_recursion::async_recursion;

#[derive(Debug, Deserialize, Clone)]
pub struct Chatter {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatterResponse {
    data: Vec<Chatter>,
    pagination: Option<Pagination>,
}

#[async_recursion]
pub async fn get_chatters(
    client_id: &str,
    bot_token: &str,
    bot_id: i32,
    channel_id: i32,
    after: &str,
    chatters: &mut Vec<Chatter>,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let mut url = format!(
        "https://api.twitch.tv/helix/chat/chatters?broadcaster_id={}&moderator_id={}&first=1000",
        channel_id, bot_id
    );

    if after != "" {
        url = format!("{}&after={}", url, after);
    }

    let res = client
        .get(&url)
        .header("Client-Id", client_id)
        .header("Authorization", format!("Bearer {}", bot_token))
        .send()
        .await?
        .json::<ChatterResponse>()
        .await?;

    chatters.extend(res.data);

    let pagination = res.pagination;
    if let Some(pagination) = pagination {
        if let Some(cursor) = pagination.cursor {
            get_chatters(
                client_id,
                bot_token,
                bot_id,
                channel_id,
                &cursor,
                chatters,
            )
            .await?;
        }
    }

    return Ok(());
}
