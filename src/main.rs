use futures_util::{SinkExt, StreamExt};
use reywen::{
    client::{methods::user::DataEditUser, Client},
    structures::{channels::message::Message, users::UserStatus},
    websocket::data::{MessageUpdateData, WebSocketEvent, WebSocketSend},
};
use reywen_txc::{
    plugins::{conf_from_file_json, federolt, pluralkit, Auth, Start},
    DB,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // import config/reywen.json

    let db = Arc::from(DB::init().await.unwrap());

    let auth: Auth = conf_from_file_json("config/reywen.json");
    let start: Start = conf_from_file_json("config/plugin_global.json");

    let client = Client::from_token(&auth.token, true).unwrap();

    // websocket

    loop {
        let (mut read, write) = client.websocket.dual_async().await;

        while let Some(input) = read.next().await {
            let write = Arc::clone(&write);
            let auth = auth.clone();
            let client = client.clone();
            let start = start.clone();
            let db = db.clone();

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
                        message_process(&client, message, &auth, start, &db).await;
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
                    } => update_message(message_id, channel_id, data, &client, start, &db).await,
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
    start: Start,
    db: &DB,
) {
    // todo bridge handling for editing messages

    // cannot check for bot made message
    if let Some(conf) = start.federolt.enabled() {
        federolt::on_message_update(message_id, channel_id, data, conf, client, db).await
    };
}

async fn message_process(client: &Client, message: Message, auth: &Auth, start: Start, db: &DB) {
    if let Some(config) = start.federolt.from_message_input(&message, auth) {
        federolt::on_message(client, &message, config, db).await;
    };

    if start.pluralkit.from_message_input(&message, auth).is_some() {
        pluralkit::on_message(client, &message, db).await;
    };
}
