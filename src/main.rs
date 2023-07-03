use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use reywen::{
    client::{methods::user::DataEditUser, Client},
    structures::{channels::message::Message, users::user::UserStatus},
    websocket::data::{MessageUpdateData, WebSocketEvent, WebSocketSend},
};
use reywen_txc::plugins::federolt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    token: String,
    bot_id: String,
    sudoers: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    // import config/reywen.json
    let file = String::from_utf8(
        std::fs::read("config/reywen.json").expect("unable to find file config/reywen.json"),
    )
    .expect("Failed to interpret byte array");

    let auth = serde_json::from_str::<Auth>(&file).expect("invalid json for Auth config");

    let client = Client::from_token(&auth.token, true).unwrap();

    // websocket

    loop {
        let (mut read, write) = client.websocket.dual_async().await;

        while let Some(input) = read.next().await {
            let write = Arc::clone(&write);
            let auth = auth.clone();
            let client = client.clone();

            tokio::spawn(async move {
                match input {
                    WebSocketEvent::Error { .. } => {}
                    WebSocketEvent::Ready { servers, .. } => {
                        client
                            .user_edit(
                                &auth.bot_id,
                                &DataEditUser::new().set_status(
                                    UserStatus::new()
                                        .set_text(&format!("servers: {}", servers.len())),
                                ),
                            )
                            .await
                            .ok();

                        write
                            .lock()
                            .await
                            .send(WebSocketSend::ping(0).into())
                            .await
                            .ok();
                    }
                    WebSocketEvent::Message { message } => {
                        message_process(&client, message, &auth).await;
                    }

                    WebSocketEvent::Pong { .. } => {
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        write
                            .lock()
                            .await
                            .send(WebSocketSend::ping(0).into())
                            .await
                            .ok();
                    }

                    WebSocketEvent::MessageUpdate {
                        message_id,
                        channel_id,
                        data,
                    } => update_message(message_id, channel_id, data, &client).await,
                    _ => {}
                };
            });
        }
    }
}

async fn update_message(
    message_id: String,
    channel_id: String,
    data: MessageUpdateData,
    client: &Client,
) {
    // todo bridge handling for editing messages

    // cannot check for bot made message
    if let Some(conf) = federolt::start().await {
        federolt::on_message_update(message_id, channel_id, data, conf, client).await
    }
}

async fn message_process(client: &Client, message: Message, auth: &Auth) {
    if message.author == auth.bot_id {
        return;
    };

    if message.content_contains("delay", " ").is_some() {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    };

    if let Some(conf) = federolt::start().await {
        federolt::on_message(client, &message, conf).await
    }
}
