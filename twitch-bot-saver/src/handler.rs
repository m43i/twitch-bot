use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::net::TcpStream;
use utils::database::entity::chat_message as chat_message_entity;
use utils::{
    parser::irc_parser::ParsedMessage,
    sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait},
    tokio_tungstenite::{MaybeTlsStream, WebSocketStream},
};

pub async fn handle_ping(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Error> {
    return utils::websocket::client::send_message("PONG :tmi.twitch.tv", ws).await;
}

pub fn handle_privmsg_save(
    msg: &ParsedMessage,
    msg_vec: &mut Vec<chat_message_entity::ActiveModel>,
) -> Result<(), Error> {
    let message = match &msg.params {
        Some(x) => String::from(x),
        None => return Err(Error::msg("No message")),
    };
    let tags = match msg.privmsg_tags() {
        Some(x) => x,
        None => return Err(Error::msg("No tags")),
    };
    let nick = String::from(&msg.source.nick);
    let channel_name = msg.command.params[0].replace("#", "").to_string();
    let naive_time = match tags.tmi_sent_ts.parse::<i64>() {
        Ok(x) => NaiveDateTime::from_timestamp_millis(x),
        Err(e) => return Err(Error::new(e)),
    };
    let time = match naive_time {
        Some(x) => DateTime::<Utc>::from_utc(x, Utc),
        None => Utc::now(),
    };

    msg_vec.push(chat_message_entity::ActiveModel {
        msg_id: ActiveValue::Set(tags.id),
        channel_id: ActiveValue::Set(tags.room_id),
        channel: ActiveValue::Set(channel_name),
        nick: ActiveValue::Set(nick),
        display_name: ActiveValue::Set(tags.display_name),
        user_id: ActiveValue::Set(tags.user_id),
        badge_info: ActiveValue::Set(tags.badge_info),
        badges: ActiveValue::Set(Some(tags.badges.join(","))),
        bits: ActiveValue::Set(tags.bits),
        color: ActiveValue::Set(tags.color),
        moderator: ActiveValue::Set(tags.moderator as i8),
        reply_msg_id: ActiveValue::Set(tags.reply_parent_msg_id),
        reply_msg_nick: ActiveValue::Set(tags.reply_parent_user_nick),
        reply_msg_display_name: ActiveValue::Set(tags.reply_parent_user_display_name),
        reply_msg_body: ActiveValue::Set(tags.reply_parent_body),
        subscriber: ActiveValue::Set(tags.subscriber as i8),
        timestamp: ActiveValue::Set(time),
        turbo: ActiveValue::Set(tags.turbo as i8),
        user_type: ActiveValue::Set(tags.user_type),
        vip: ActiveValue::Set(tags.vip as i8),
        admin: ActiveValue::Set(tags.admin as i8),
        body: ActiveValue::Set(message),
        emotes: ActiveValue::Set(tags.emotes),
        ..Default::default()
    });

    return Ok(());
}

pub async fn handle_clearmsg_update(
    msg: &ParsedMessage,
    msg_vec: &mut Vec<chat_message_entity::ActiveModel>,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    let tags = match msg.clearmsg_tags() {
        Some(x) => x,
        None => return Err(Error::msg("No tags")),
    };

    match msg_vec
        .iter_mut()
        .find(|x| x.msg_id == ActiveValue::Set(tags.target_msg_id.to_string()))
    {
        Some(x) => {
            x.deleted = ActiveValue::Set(1);
            x.deleted_timestamp = ActiveValue::Set(Some(Utc::now()));
            x.updated_at = ActiveValue::Set(
                NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0).unwrap(),
            );
        }
        None => {
            let chat_message = chat_message_entity::Entity::find_by_id(tags.target_msg_id)
                .one(db)
                .await?;

            let chat_message = match chat_message {
                Some(x) => x,
                None => return Err(Error::msg("No chat message")),
            };

            let mut chat_message: chat_message_entity::ActiveModel = chat_message.into();

            chat_message.deleted = ActiveValue::Set(1);
            chat_message.deleted_timestamp = ActiveValue::Set(Some(Utc::now()));
            chat_message.updated_at = ActiveValue::Set(
                NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0).unwrap(),
            );

            chat_message.update(db).await?;
        }
    };

    return Ok(());
}
