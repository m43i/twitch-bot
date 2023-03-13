use anyhow::{Error, Result};

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
struct ValidationRes {
    client_id: String,
    login: String,
    scopes: Vec<String>,
    user_id: String,
    expires_in: u64,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    loop {
        let mut con = cache::connect(&redis_url).await?;
        let tokens = auth::token::get_all_active_tokens(&mut con).await?;

        for token in tokens {
            auth::token::validate_token(&token).await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(50 * 60)).await;
    }
}
