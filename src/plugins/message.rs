use serde::{Deserialize, Serialize};

use reywen::{
    delta::{
        fs::fs_to_str,
        lreywen::{convec, crash_condition},
        oop::Reywen,
    },
    quark::delta::message::RMessage,
};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageConf {
    pub enabled: bool,
}

// main message engine
pub async fn message_main(client: &Reywen, input_message: &RMessage) {
    let client: Reywen = client.to_owned();

    // import config
    let conf = fs_to_str("config/message.json").expect("failed to read config/message.json\n{e}");

    let message_conf: MessageConf =
        serde_json::from_str(&conf).expect("Failed to deser message.json");

    // return if this plugin is disabled
    if !message_conf.enabled {
        return;
    };

    // covers vector crash conditions
    crash_condition(input_message, None);

    // content vector
    let convec = convec(input_message);

    let mes = match convec[0] as &str {
        "?Mog" | "?mog" => ":01G7MT5B978E360NB6VWAS9SJ6:",
        "?ver" | "?version" => {
            "Reywen is rolling release, there is no release numbers only commits :trol:"
        }
        _ => "",
    };
    // if applicable, send
    if !mes.is_empty() {
        client.sender(mes).await;
    };
}
