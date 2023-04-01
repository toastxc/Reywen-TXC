// external crates
use futures_util::StreamExt;
use reywen::{
    client::Do,
    structs::{auth::Auth, message::Message},
    websocket::Websocket,
};

// imported from reywen library



#[tokio::main]
async fn main() {
    // import config/reywen.json
    let file = String::from_utf8(
        std::fs::read("config/reywen.json").expect("unable to find file config/reywen.json"),
    )
    .expect("Failed to interpret byte array");

    // Deserilize reywen.json Websocket and Auth
    let websocket =
        serde_json::from_str::<Websocket>(&file).expect("invalid json for websocket config");

    let auth = serde_json::from_str::<Auth>(&file).expect("invalid json for Auth config");

    // restart websocekt - always
    loop {
        websocket
            .clone()
            .generate()
            .await
            // from websocket config establish a connection
            .for_each(|message| async {
                // for every message
                if let Ok(raw_message) = message {
                    // if the message is valid
                    if let Ok(input_message) =
                        serde_json::from_str::<Message>(&raw_message.into_text().unwrap())
                    // and of type Message
                    {
                        let client = Do::new(&auth, &input_message);
                        // spawn a new task
                        tokio::spawn(message_process(client));
                    }
                }
            })
            .await;
    }
}

async fn message_process(client: Do) {
    tokio::join!(
        plugins::bridge::br_main(&client),
        plugins::e6::e6_main(&client),
        plugins::message::message_main(&client),
        plugins::plural::plural_main(&client),
        plugins::tomo::t_main(&client),
        plugins::shell::shell_main(&client),
    );
}

// basic CLI tool for checking content
pub fn crash_condition(input_message: &Message, character: Option<&str>) -> bool {
    if input_message.content.is_none() {
        return true;
    };

    let temp_convec: Vec<&str> = input_message
        .content
        .as_ref()
        .unwrap()
        .split(' ')
        .collect::<Vec<&str>>();

    let mut length = 2;

    if character.is_none() {
        length = 1;
    };

    if temp_convec.len() < length {
        return true;
    };

    if character.is_some() && character != Some(temp_convec[0]) {
        return true;
    };
    false
}

// Simple markdown formating
pub fn md_fmt(message: &str, emoji: RE) -> String {
    let emoji = RE::e(emoji);

    format!("{emoji} $\\color{{grey}}\\small\\textsf{{{message}}}$")
}

// Custom emojis
pub enum RE {
    Search,
    Send,
    Rm,
    Json,
    Insert,
}

// Serilizing emojis
impl RE {
    pub fn e(self) -> String {
        match self {
            RE::Search => ":01GQE862YPERANAJC30GNKH625:",
            RE::Send => ":01GQE848SKP794SKZYY8RTCXF1:",
            RE::Rm => ":01GQE86CT9MKAHPTG55HMTG7TR:",
            RE::Json => ":01GQE86K0CG3FWA0D6FRY7JT0R:",
            RE::Insert => ":01GQE86SAYFDXZE2F39YHJMB1F:",
        }
        .to_string()
    }
}
pub fn lte(input: &str) -> String {
    format!("[]({input})")
}
