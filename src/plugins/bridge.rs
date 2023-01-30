use reywen::{
    client::Do,
    structs::message::{DataMessageSend, Masquerade, Reply},
};
use serde::{Deserialize, Serialize};

use crate::crash_condition;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrConf {
    pub enabled: bool,
    pub channel_1: String,
    pub channel_2: String,
}

pub async fn br_main(client: &Do) {
    // bridge is a very unconventional use of Reywen and as such has special requirements
    // client is mutated, and input_message is directly accessed
    // this plugin may be a bad example of normal usage of Reywen
    let mut client: Do = client.to_owned();

    // import config from file
    let conf: BrConf = serde_json::from_str(
        &String::from_utf8(
            std::fs::read("config/bridge.json").expect("failed to read config/message.json\n{e}"),
        )
        .unwrap(),
    )
    .expect("invalid config");

    // fail conditions
    if !conf.enabled {
        return;
    };
    crash_condition(&client.input_message, None);

    // removes the chance of bot feedback loop
    if client.auth.bot_id.is_empty() {
        println!("WARN: bot ID is empty, this can lead to undefined behavior (bridge)");
        return;
    };
    if client.author_is_bot() && client.masquerade().is_some() {
        return;
    };

    // channel matcher
    let chan_rec = match (
        client.channel_is(&conf.channel_1),
        client.channel_is(&conf.channel_2),
    ) {
        (true, _) => conf.channel_2,
        (_, true) => conf.channel_1,
        _ => return,
    };

    // modifying input message - makes reywen send a message in a different channel to that of websocket
    client.input_message.channel = chan_rec;

    // if a profile is already using masquerade, copy it
    // otherwise generate a masq profile based on their details
    let br_masq = match client.masquerade() {
        None => masq_from_user(&client).await,
        Some(a) => a,
    };

    // converts replies from websocket to API structure
    let mut replies: Vec<Reply> = Vec::new();
    for x in client.replies().unwrap_or_default() {
        let reply = Reply::new().id(&x);
        replies.push(reply);
    }
    // custom message payload
    let payload = DataMessageSend::new()
        .content(&client.content())
        .masquerade(br_masq)
        .replies(replies);

    client.send(payload).await;
}

// this method is very slow as it calls API several times, but it is safer than the old method
async fn masq_from_user(client: &Do) -> Masquerade {
    let user = client.self_fetch().await;

    if let Some(user) = user {
        let avatar = client.self_fetch_avatar().await.unwrap_or(String::from(
            "https://api.revolt.chat/users/01FYZHW3KFZ5QN8R3KCQ8JH79R/default_avatar",
        ));
        return Masquerade::new().name(&user.username).avatar(&avatar);
    };
    // on failure masquerade is empty
    Masquerade::new()
}
