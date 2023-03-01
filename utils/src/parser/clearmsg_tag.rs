use std::collections::HashMap;
use anyhow::{Result, Error};

#[derive(Debug, Clone)]
pub struct ClearMsgTags {
    pub login: String,
    pub room_id: Option<i64>,
    pub target_msg_id: String,
    pub tmi_sent_ts: i64,
}

/**
 * Parse the tags from a CLEARMSG message
 */
pub fn parse(tags: &HashMap<String, String>) -> Result<ClearMsgTags, Error> {
    let login = match tags.get("login") {
        Some(x) => x.to_string(),
        _ => return Err(Error::msg("Missing login tag")),
    };

    let room_id = match tags.get("room-id") {
        Some(x) => match x.parse::<i64>() {
            Ok(x) => Some(x),
            Err(_) => None,
        },
        _ => return Err(Error::msg("Missing room-id tag")),
    };

    let target_msg_id = match tags.get("target-msg-id") {
        Some(x) => x.to_string(),
        _ => return Err(Error::msg("Missing target-msg-id tag")),
    };

    let tmi_sent_ts = match tags.get("tmi-sent-ts") {
        Some(x) => x.parse::<i64>()?,
        _ => return Err(Error::msg("Missing tmi-sent-ts tag")),
    };

    Ok(ClearMsgTags {
        login,
        room_id,
        target_msg_id,
        tmi_sent_ts,
    })
}
