use anyhow::{Error, Result};
use cache::redis::aio::Connection;
use database::sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Request error")]
    RequestError,
    #[error("Token expired")]
    Expired,
    #[error("Token invalid")]
    Invalid,
}

#[derive(Debug, Serialize)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i32,
    scope: Vec<String>,
    token_type: String,
}

/**
 * Get a bot access token for a given bot
 */
pub async fn get_bot_token(
    bot: &str,
    client_id: &str,
    client_secret: &str,
    db: &DatabaseConnection,
    redis: &mut Connection,
) -> Result<String, Error> {
    let token = cache::get::<String>(&format!("bot:{}", bot), redis).await?;
    if token.is_some() {
        let token = token.unwrap();
        return Ok(token);
    }

    let token = refresh_bot_token(bot, client_id, client_secret, db, redis).await?;
    return Ok(token);
}

/**
 * Refresh a bot access token for a given bot
 */
pub async fn refresh_bot_token(
    bot: &str,
    client_id: &str,
    client_secret: &str,
    db: &DatabaseConnection,
    redis: &mut Connection,
) -> Result<String, Error> {
    let token = database::handler::token::get_bot_refresh_token(bot, db).await?;
    let res = refresh_token(&token, client_id, client_secret).await?;

    database::handler::token::update_bot_refresh_token(db, &res.refresh_token).await?;
    cache::set_with_ttl(
        &format!("bot:{}", bot),
        &res.access_token,
        (&res.expires_in - 60 * 20) as usize,
        redis,
    )
    .await?;
    return Ok(res.access_token);
}

/**
 * Refresh a token against the twitch api endpoint
 */
async fn refresh_token(
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<RefreshTokenResponse, AuthError> {
    let client = reqwest::Client::new();

    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&RefreshTokenRequest {
            grant_type: String::from("refresh_token"),
            refresh_token: String::from(refresh_token),
            client_id: String::from(client_id),
            client_secret: String::from(client_secret),
        })
        .send()
        .await;

    let res = match res {
        Ok(res) => res,
        Err(_) => return Err(AuthError::RequestError),
    };

    return match res.status() {
        reqwest::StatusCode::BAD_REQUEST => Err(AuthError::Invalid),
        reqwest::StatusCode::UNAUTHORIZED => Err(AuthError::Expired),
        reqwest::StatusCode::OK => {
            let res = res.json::<RefreshTokenResponse>().await;
            let res = match res {
                Ok(res) => res,
                Err(_) => return Err(AuthError::RequestError),
            };

            return Ok(res);
        }
        _ => Err(AuthError::RequestError),
    };
}
