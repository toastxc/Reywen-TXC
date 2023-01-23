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
