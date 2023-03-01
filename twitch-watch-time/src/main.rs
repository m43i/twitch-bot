use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ChatterList {
    broadcaster: Vec<String>,
    vips: Vec<String>,
    moderators: Vec<String>,
    staff: Vec<String>,
    admins: Vec<String>,
    global_mods: Vec<String>,
    viewers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chatters {
    chatter_count: u32,
    chatters: ChatterList,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let client = reqwest::Client::new();

    loop {
        let res = client
            .get("https://tmi.twitch.tv/group/user/anniislost/chatters")
            .send()
            .await?
            .json::<Chatters>()
            .await?;

        println!("{:?}", res);

        tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
    }
}
