// external crates
use futures_util::StreamExt;
use reywen::{
    client::Do,
    structs::{auth::Auth, message::Message},
    websocket::Websocket,
};

use reywen_txc::plugins::*;

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
        bridge::br_main(&client),
        e6::e6_main(&client),
        message::message_main(&client),
        plural::plural_main(&client),
        tomo::t_main(&client),
        shell::shell_main(&client),
     //   ticket::main(client: &Do),
    );
}
