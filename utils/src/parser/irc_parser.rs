use anyhow::{Error, Result};
use std::collections::HashMap;

use crate::parser::privmsg_tag::PrivMsgTags;
use crate::parser::clearmsg_tag::ClearMsgTags;

#[derive(Debug, Clone)]
pub struct ChatSource {
    pub nick: String,
    pub host: String,
}

#[derive(Debug, Clone)]
pub enum IRCCommandType {
    PING,
    PRIVMSG,
    CLEARMSG,
    JOIN,
    UNKNOWN,
}

#[derive(Debug, Clone)]
pub struct IRCCommand {
    pub command: IRCCommandType,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChatCommand {
    pub command: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub tags: Option<HashMap<String, String>>,
    pub source: ChatSource,
    pub command: IRCCommand,
    pub params: Option<String>,
    pub chat_command: Option<ChatCommand>,
}

impl ParsedMessage {
    pub fn privmsg_tags(&self) -> Option<PrivMsgTags> {
        let tags = match self.tags.clone() {
            Some(x) => x,
            _ => return None,
        };

        let tags = crate::parser::privmsg_tag::parse(&tags);

        match tags {
            Ok(x) => Some(x),
            _ => None,
        }
    }

    pub fn clearmsg_tags(&self) -> Option<ClearMsgTags> {
        let tags = match self.tags.clone() {
            Some(x) => x,
            _ => return None,
        };

        let tags = crate::parser::clearmsg_tag::parse(&tags);


        match tags {
            Ok(x) => Some(x),
            _ => None,
        }
    }
}

/**
 * Parse the source of the message from the twitch irc
 */
pub async fn parse(msg: &str) -> Result<ParsedMessage, Error> {
    let mut idx = 0;
    let mut tags: Option<HashMap<String, String>> = None;
    let mut source: ChatSource = ChatSource {
        nick: "".to_string(),
        host: "".to_string(),
    };

    if msg.chars().nth(idx) == Some('@') {
        idx += 1;
        let end = match msg[idx..].find(" ") {
            Some(x) => x + idx,
            _ => return Err(Error::msg("Invalid message")),
        };
        let tags_string = msg[idx..end].split(";").collect::<Vec<&str>>();
        tags = Some(HashMap::new());
        for tag in tags_string {
            let tag = tag.split("=").collect::<Vec<&str>>();
            tags.as_mut()
                .unwrap()
                .insert(tag[0].to_string(), tag[1].to_string());
        }

        idx = end + 1;
    }

    if msg.chars().nth(idx) == Some(':') {
        idx += 1;
        let end = match msg[idx..].find(" ") {
            Some(x) => x + idx,
            _ => return Err(Error::msg("Invalid message")),
        };
        source = parse_source(&msg[idx..end].to_string());
        idx = end + 1;
    }

    let end = match msg[idx..].find(":") {
        Some(x) => x + idx,
        _ => msg.len(),
    };

    let command = parse_command(&msg[idx..end].trim().to_string());
    let mut params: Option<String> = None;
    let mut chat_command: Option<ChatCommand> = None;

    if end != msg.len() {
        idx = end + 1;

        params = Some(msg[idx..].to_string());
        chat_command = parse_params(&msg[idx..].to_string());
    }

    return Ok(ParsedMessage {
        tags,
        source,
        command,
        params,
        chat_command,
    });
}

/**
 * Parse the command from the twitch irc
 */
fn parse_command(command: &str) -> IRCCommand {
    let split = command.split(" ").collect::<Vec<&str>>();
    let mut params: Vec<String> = Vec::new();

    for param in split[1..].iter() {
        params.push(param.to_string());
    }

    return IRCCommand {
        command: match split[0] {
            "PING" => IRCCommandType::PING,
            "PRIVMSG" => IRCCommandType::PRIVMSG,
            "JOIN" => IRCCommandType::JOIN,
            "CLEARMSG" => IRCCommandType::CLEARMSG,
            _ => IRCCommandType::UNKNOWN
        },
        params,
    };
}

/**
 * Parse the chat command from the twitch irc
 */
fn parse_params(params: &str) -> Option<ChatCommand> {
    if !params.starts_with("!") {
        return None;
    }

    let split = params.split(" ").collect::<Vec<&str>>();
    let command = split[0].replace("!", "").to_lowercase();
    let args = split[1..].iter().map(|x| x.to_string()).collect();

    return Some(ChatCommand {
        command,
        params: args,
    });
}

/**
 * Parse the source of the message from the twitch irc
 */
fn parse_source(prefix: &str) -> ChatSource {
    let split = prefix.split("!").collect::<Vec<&str>>();

    if split.len() == 2 {
        return ChatSource {
            nick: split[0].to_string(),
            host: split[1].to_string(),
        };
    } else {
        return ChatSource {
            nick: "".to_string(),
            host: split[0].to_string(),
        };
    }
}
