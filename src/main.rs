// external packages
use futures_util::StreamExt;
// plugins
pub mod plugins {
    pub mod bridge;
    pub mod e6;
    pub mod message;
    pub mod plural;
    pub mod shell;
    pub mod tomo;
}

// reywen lib
use reywen::{
    bonfire::bonfire::from_ws,
    delta::{
        delta::rev_message_in,
        fs::{conf_init, ws_init},
        oop::Reywen,
    },
};

use crate::plugins::{
    bridge::br_main, e6::e6_main, message::message_main, plural::plural_main, shell::shell_main,
    tomo::t_main,
};

#[tokio::main]
async fn main() {
    let auth = conf_init().unwrap();
    println!("booting...");

    let ws = ws_init().unwrap();

    println!("websocket established");

    ws.clone()
        .generate()
        .await
        .for_each(|message_in| async {
            let message = from_ws(message_in);

            let input_message = rev_message_in(message);

            let input_message = match input_message {
                Err(_) => return,
                Ok(a) => a,
            };
            let client = Reywen::new(auth.clone(), &input_message);

            tokio::join!(
                br_main(&client, &input_message),
                e6_main(&client, &input_message),
                message_main(&client, &input_message),
                plural_main(&client, &input_message),
                t_main(&client, &input_message),
                shell_main(&client, &input_message),
            );
        })
        .await;
}

// i dont like markdown
pub fn md_fmt(mes: &str) -> String {
    format!("- $\\color{{grey}}\\small\\textsf{{{mes}}}$")
}