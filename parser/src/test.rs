#[tokio::test]
async fn ping_parse_test() {
    let input = "PING :tmi.twitch.tv";
    let parsed = crate::irc_parser::parse(input).await;

    assert_eq!(parsed.is_ok(), true);
    let parsed = parsed.unwrap();

    assert_eq!(
        parsed.command.command,
        crate::irc_parser::IRCCommandType::PING
    );
    assert_eq!(parsed.params, Some("tmi.twitch.tv".to_string()));
}

#[tokio::test]
async fn privmsg_parse_test() {
    let input = "@badges=staff/1,broadcaster/1,turbo/1;color=#FF0000;display-name=PetsgomOO;emote-only=1;emotes=33:0-7;flags=0-7:A.6/P.6,25-36:A.1/I.2;id=c285c9ed-8b1b-4702-ae1c-c64d76cc74ef;mod=0;room-id=81046256;subscriber=0;turbo=0;tmi-sent-ts=1550868292494;user-id=81046256;user-type=staff :petsgomoo!petsgomoo@petsgomoo.tmi.twitch.tv PRIVMSG #petsgomoo :DansGame";

    let parsed = crate::irc_parser::parse(input).await;
    assert_eq!(parsed.is_ok(), true);
    let parsed = parsed.unwrap();

    let tags = parsed.privmsg_tags();
    assert_eq!(tags.is_some(), true);
    let tags = tags.unwrap();

    assert_eq!(
        parsed.command.command,
        crate::irc_parser::IRCCommandType::PRIVMSG
    );
    assert_eq!(parsed.params, Some("DansGame".to_string()));
    assert_eq!(parsed.source.nick, "petsgomoo");
    assert_eq!(parsed.source.host, "petsgomoo@petsgomoo.tmi.twitch.tv");
    assert_eq!(tags.admin, true);
    assert_eq!(tags.room_id, 81046256);
}
