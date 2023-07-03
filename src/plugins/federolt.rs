use std::collections::HashMap;

use bson::doc;

use futures_util::future::join_all;
use mongodb::{Collection, Database};
use reywen::{
    client::{
        methods::message::{DataEditMessage, DataMessageSend},
        Client,
    },
    structures::channels::message::{Masquerade, Message, Reply, SendableEmbed},
    websocket::data::MessageUpdateData,
};

use serde::{Deserialize, Serialize};

use super::conf_from_file;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FederoltGroups {
    enable: bool,
    groups: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessageAlias {
    // message, channel
    pub message_ids: HashMap<String, String>,
    pub origin_message: String,
}

impl MessageAlias {
    pub fn new(origin_message: String) -> Self {
        Self {
            origin_message,
            ..Default::default()
        }
    }
    pub fn add_message(&mut self, message_id: String, channel_id: String) -> Self {
        self.message_ids.insert(message_id, channel_id);
        self.to_owned()
    }
}

pub async fn start() -> Option<FederoltGroups> {
    let conf: FederoltGroups = conf_from_file("config/federolt.toml");
    if !conf.enable {
        None
    } else {
        Some(conf)
    }
}

pub struct DB {}
impl DB {
    pub async fn alias() -> Result<Collection<MessageAlias>, mongodb::error::Error> {
        Ok(Self::db().await?.collection("alias"))
    }
    pub async fn db() -> Result<Database, mongodb::error::Error> {
        easymongo::mongo::Mongo::new()
            .username("username")
            .password("password")
            .database("test")
            .db_generate()
            .await
    }
}

pub fn edited_messages(groups: &FederoltGroups, target_channel: &str) -> Option<Vec<String>> {
    let mut channel_vec = Vec::new();
    groups.groups.iter().for_each(|group| {
        if group.1.contains(&String::from(target_channel)) {
            for channel in group.1 {
                if !channel_vec.contains(channel) && channel != target_channel {
                    channel_vec.push(channel.to_owned())
                }
            }
        }
    });
    if channel_vec.is_empty() {
        None
    } else {
        Some(channel_vec)
    }
}

pub async fn on_message_update(
    message_id: String,
    channel_id: String,
    data: MessageUpdateData,
    conf: FederoltGroups,
    client: &Client,
) {
    if edited_messages(&conf, &channel_id).is_some() {
        let db = DB::alias().await.unwrap();

        let mut new_message = DataEditMessage::new();
        if let Some(content) = data.content {
            new_message.content(&content);
        };

        let client = client.to_owned();

        if let Some(group) = db
            .find_one(doc!("origin_message": message_id), None)
            .await
            .unwrap()
        {
            for (message_id, channel_id) in group.message_ids {
                let client = client.clone();
                let new_message = new_message.clone();
                tokio::spawn(async move {
                    client
                        .message_edit(&channel_id, &message_id, &new_message)
                        .await
                        .ok();
                });
            }
        }
    }
}

pub async fn on_message(client: &Client, message: &Message, conf: FederoltGroups) {
    // messaging sending data
    let mut message_send_index = Vec::new();

    // if message is a part of a group
    if let Some(channels) = edited_messages(&conf, &message.channel) {
        // convert message
        let new_message = message_from_input(client, message).await;

        // for every channel in groups send message and save results
        for channel in channels {
            let client = client.clone();
            let new_message = new_message.clone();
            message_send_index.push(tokio::spawn(async move {
                client.message_send(&channel, &new_message).await
            }));
        }
    };
    // spawn all messages
    let message_send_index = join_all(message_send_index).await;
    // database
    let db = DB::alias().await.unwrap();

    let mut message_db = MessageAlias::new(message.id.to_owned());

    // for every message thats valid save the IDs
    message_send_index.into_iter().for_each(|item| {
        if let Ok(Ok(Message { id, channel, .. })) = item {
            message_db.add_message(id, channel);
        };
    });

    db.insert_one(message_db, None).await.unwrap();
}

async fn message_from_input(client: &Client, message: &Message) -> DataMessageSend {
    let mut new_message = DataMessageSend::new();
    new_message.content = message.content.to_owned();
    new_message.set_masquerade(&masq_from_message(client, message).await);

    if let Some(replies) = message.replies.as_ref() {
        new_message.replies = Some(
            replies
                .iter()
                .map(|reply| Reply {
                    id: reply.to_owned(),
                    mention: false,
                })
                .collect(),
        );
    };

    if let Some(embeds) = message.embeds.as_ref() {
        new_message.embeds = Some(
            embeds
                .iter()
                .map(|embed| Into::<SendableEmbed>::into(embed.to_owned()))
                .collect(),
        );
    }

    if let Some(attachments) = message.attachments.as_ref() {
        for attachment in attachments {
            let hide = format!(
                "[](https://autumn.revolt.chat/attachments/{})",
                attachment.id
            );
            if new_message.content.is_none() {
                new_message.set_content(&hide);
            } else {
                new_message.content = Some(new_message.content.unwrap() + &hide);
            }
        }
    };

    if !&message.interactions.is_default() {
        new_message.interactions = Some(message.interactions.clone())
    }

    new_message
}

async fn masq_from_message(client: &Client, message: &Message) -> Masquerade {
    let mut newmasq = Masquerade::new();

    if let Some(masq) = message.masquerade.clone() {
        if masq.name.is_some() && masq.avatar.is_some() {
            newmasq.name = masq.name;
            newmasq.avatar = masq.avatar;
        };

        if masq.colour.is_some() {
            newmasq.colour = masq.colour;
        }
    } else {
        let user = client.user_fetch(&message.author).await.unwrap();
        newmasq.set_name(&user.username);

        newmasq.avatar = user
            .avatar
            .as_ref()
            .map(|a| format!("https://autumn.revolt.chat/avatars/{}", a.id));
    }

    newmasq
}
