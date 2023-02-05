use reywen::client::Do;
use serde::{Deserialize, Serialize};

use crate::{crash_condition, md_fmt, RE};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageConf {
    pub enabled: bool,
}

// main message engine
pub async fn message_main(client: &Do) {
    let help = format!(
        "### Reywen-TXC\n{} {}\n{} {}\n{} {}\n{} {}\n{} {}\n{} {}",
        md_fmt("?e", RE::Send),
        "E621 Interaction",
        md_fmt("?p", RE::Send),
        "Masquerade Utility",
        md_fmt("?/", RE::Send),
        "BASH SHell",
        md_fmt("?t", RE::Send),
        "Tomogatchi Game",
        md_fmt("?mog", RE::Send),
        "Amogus",
        md_fmt("?ver", RE::Send),
        "Displays version"
    );

    // import config

    let conf: MessageConf = serde_json::from_str(
        &String::from_utf8(
            std::fs::read("config/message.json").expect("failed to read config/message.json\n{e}"),
        )
        .unwrap(),
    )
    .expect("invalid config");

    // return if this plugin is disabled
    if !conf.enabled {
        return;
    };

    // covers vector crash conditions
    crash_condition(&client.input_message, None);

    // match the first word from websocket
    let mes = match &client.input().convec()[0] as &str {
        "?Mog" | "?mog" => ":01G7MT5B978E360NB6VWAS9SJ6:",
        "?ver" | "?version" => "Reywen.rs 0.1.4",
        "?help" => "help",
        _ => "",
    };

    // if applicable, send
    if mes == "help" {
        client.message().sender(&help).await;
    } else if !mes.is_empty() {
        client.message().sender(mes).await;
    }
}
