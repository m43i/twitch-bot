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
    let client = reqwest::Client::new();
    let token = std::env::var("TWITCH_TOKEN").expect("TWITCH_TOKEN not set");
    loop {
        let res = client
            .get("https://id.twitch.tv/oauth2/validate")
            .header("Authorization", format!("OAuth {}", token))
            .send()
            .await;
        let res = match res {
            Ok(x) => x,
            Err(err) => {
                println!("Error: {}", err);
                continue;
            }
        };

        let res = res.json::<ValidationRes>().await;
        let res = match res {
            Ok(x) => x,
            Err(err) => {
                println!("Error: {}", err);
                continue;
            }
        };

        println!("{:?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(50 * 60)).await;
    }
}
