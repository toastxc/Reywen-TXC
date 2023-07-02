use std::collections::HashMap;

use reywen::{
    client::{methods::message::DataMessageSend, Client},
    structures::channels::message::{Masquerade, Message, Reply, SendableEmbed},
};

use serde::{Deserialize, Serialize};

use super::conf_from_file;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FederoltGroups {
    enable: bool,
    groups: HashMap<String, Vec<String>>,
}

pub async fn start() -> Option<FederoltGroups> {
    let conf: FederoltGroups = conf_from_file("config/federolt.toml");
    if !conf.enable {
        return None;
    } else {
        return Some(conf);
    }
}

pub async fn on_message(client: &Client, message: &Message, conf: FederoltGroups) {
    let mut watcher = Vec::new();

    for (_, mut group) in conf.groups {
        if !group.contains(&message.channel) {
            continue;
        } else {
            group.retain(|target_channel| target_channel != &message.channel);
        };

        let new_message = message_from_input(client, message).await;

        for channel in group {
            if watcher.contains(&channel) {
            } else {
                watcher.push(channel.clone());

                client.message_send(&channel, &new_message).await.ok();
            }
        }
    }
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
                .into_iter()
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
