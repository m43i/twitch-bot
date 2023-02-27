use std::collections::HashMap;
use anyhow::{Error, Result};

use crate::database::entity::sea_orm_active_enums::UserType;

#[derive(Debug, Clone)]
pub struct PrivMsgTags {
    pub badge_info: Option<String>,
    pub badges: Vec<String>,
    pub admin: bool,
    pub bits: i32,
    pub color: String,
    pub display_name: String,
    pub emotes: Option<String>,
    pub id: String,
    pub moderator: bool,
    pub reply_parent_msg_id: Option<String>,
    pub reply_parent_user_nick: Option<String>,
    pub reply_parent_user_display_name: Option<String>,
    pub reply_parent_body: Option<String>,
    pub room_id: i32,
    pub subscriber: bool,
    pub tmi_sent_ts: String,
    pub turbo: bool,
    pub user_id: i32,
    pub user_type: UserType,
    pub vip: bool,
}

pub fn parse(tags: &HashMap<String, String>) -> Result<PrivMsgTags, Error> {
    let badge_info = match tags.get("badge-info") {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let badges = match tags.get("badges") {
        Some(x) => x.split(",").map(|x| x.to_string()).collect::<Vec<String>>(),
        None => return Err(Error::msg("No badges")),
    };

    let admin = badges.iter().any(|x| x.contains("broadcaster/1"));

    let bits = match tags.get("bits") {
        Some(x) => match x.parse::<i32>() {
            Ok(x) => x,
            Err(_) => 0,
        },
        None => 0,
    };

    let color = match tags.get("color") {
        Some(x) => x.to_string(),
        None => return Err(Error::msg("No color")),
    };

    let display_name = match tags.get("display-name") {
        Some(x) => x.to_string(),
        None => return Err(Error::msg("No display name")),
    };

    let emotes = match tags.get("emotes") {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let id = match tags.get("id") {
        Some(x) => x.to_string(),
        None => return Err(Error::msg("No id")),
    };

    let moderator = match tags.get("mod") {
        Some(x) => match x.as_str() {
            "1" => true,
            _ => false,
        },
        None => return Err(Error::msg("No mod")),
    };

    let reply_parent_msg_id = match tags.get("reply-parent-msg-id") {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let reply_parent_user_nick = match tags.get("reply-parent-user-login") {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let reply_parent_user_display_name = match tags.get("reply-parent-display-name") {
        Some(x) => Some(x.to_string()),
        None => None,
    };

    let reply_parent_body = match tags.get("reply-parent-msg-body") {
        Some(x) => Some(x.to_string().replace("\\s", " ")),
        None => None,
    };

    let room_id = match tags.get("room-id") {
        Some(x) => match x.parse::<i32>() {
            Ok(x) => x,
            Err(_) => return Err(Error::msg("Invalid room id")),
        },
        None => return Err(Error::msg("No room id")),
    };

    let subscriber = match tags.get("subscriber") {
        Some(x) => match x.as_str() {
            "1" => true,
            _ => false,
        },
        None => return Err(Error::msg("No subscriber")),
    };

    let tmi_sent_ts = match tags.get("tmi-sent-ts") {
        Some(x) => x.to_string(),
        None => return Err(Error::msg("No tmi-sent-ts")),
    };

    let turbo = match tags.get("turbo") {
        Some(_) => true,
        None => false,
    };

    let user_type = match tags.get("user-type") {
        Some(x) => match x.as_str() {
            "global_mod" => UserType::Globalmod,
            "admin" => UserType::Globaladmin,
            "staff" => UserType::Staff,
            _ => UserType::Normal,
        },
        None => UserType::Normal,
    };

    let user_id = match tags.get("user-id") {
        Some(x) => match x.parse::<i32>() {
            Ok(x) => x,
            Err(_) => return Err(Error::msg("Invalid user id")),
        },
        None => return Err(Error::msg("No user id")),
    };

    let vip = match tags.get("vip") {
        Some(_) => true,
        None => false,
    };

    return Ok(PrivMsgTags {
        badge_info,
        badges,
        admin,
        bits,
        color,
        display_name,
        emotes,
        id,
        moderator,
        reply_parent_msg_id,
        reply_parent_user_nick,
        reply_parent_user_display_name,
        reply_parent_body,
        room_id,
        subscriber,
        tmi_sent_ts,
        turbo,
        user_id,
        user_type,
        vip,
    });
}
