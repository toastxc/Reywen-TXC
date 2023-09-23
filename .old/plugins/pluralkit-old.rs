use std::{collections::HashMap, ops::Index};

use bson::doc;
use mongodb::Collection;
use reywen::{
    client::{methods::message::DataMessageSend, Client},
    structures::channels::message::{Masquerade, Message},
};
use serde::{Deserialize, Serialize};

use crate::DB;
use indexmap::IndexSet;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct PluralUser {
    pub user_id: String,
    pub profiles: RealMasquerade,
}

pub type RealMasquerade = HashMap<String, MasqueradeData>;
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MasqueradeData {
    pub name: String,
    pub avatar: Option<String>,
    pub color: Option<String>,
    pub aliases: Option<IndexSet<String>>,
}

impl PluralUser {
    pub fn insert_profile(&mut self, data: &MasqueradeData) -> Self {
        self.profiles.insert(data.name.clone(), data.clone());
        self.to_owned()
    }
}

impl From<MasqueradeData> for Masquerade {
    fn from(val: MasqueradeData) -> Self {
        Masquerade {
            name: Some(val.name),
            avatar: val.avatar,
            colour: val.color,
        }
    }
}

impl From<Masquerade> for MasqueradeData {
    fn from(value: Masquerade) -> Self {
        MasqueradeData {
            name: value.name.unwrap_or(String::from("NULLNAME")),
            avatar: value.avatar,
            color: value.colour,
            aliases: None,
        }
    }
}

const COL: &str = "pluralkit";

impl PluralUser {
    pub fn check_alias(&self, alias: &str) -> Option<MasqueradeData> {
        for (_, data) in self.profiles.clone().into_iter() {
            if data
                .clone()
                .aliases
                .is_some_and(|aliases| aliases.contains(alias))
            {
                return Some(data.clone());
            }
        }

        None
    }
}

impl MasqueradeData {
    pub fn data_override(&mut self, new: MasqueradeData) -> Self {
        self.avatar = match new.avatar.map(|a| a.to_lowercase()).as_deref() {
            Some("none") => None,
            Some("skip") => self.avatar.clone(),
            Some(other) => Some(other.to_owned()),
            None => None,
        };

        self.color = match new.color.map(|a| a.to_lowercase()).as_deref() {
            Some("none") => None,
            Some("skip") => self.color.clone(),
            Some(other) => Some(other.to_owned()),
            None => None,
        };

        self.to_owned()
    }
}

const HELP: &str = "HAVENT MADDE A HELP MESSAGE YET UWU";
pub async fn on_message(client: &Client, message: &Message) {
    if let Some(mut profiles) = message.content_contains("?p", " ") {
        profiles.remove(0);

        if profiles.len() < 2 {
            client
                .message_send(&message.channel, &DataMessageSend::new().set_content(HELP))
                .await
                .ok();
            return;
        };

        match profiles.index(0).as_str() {
            "profile" => profiles_fn(client, message).await,
            "alias" => alias(client, message).await,
            // alias trigger
            _ => alias_trigger(client, message).await,
        }
    };
}

async fn alias_trigger(client: &Client, message: &Message) {
   
    let mut content = message.clone().content.into_iter().collect::<Vec<String>>();
    content.remove(0);

    if let Ok(Some(user)) = user_find(&db, &message.author).await {
        if let Some(a) = user.check_alias(&content[1]) {
            content.remove(0);

            client
                .message_send(
                    &message.channel,
                    &DataMessageSend::new()
                        .set_content(&content_vec(content))
                        .set_masquerade(&a.into()),
                )
                .await
                .ok();

            client
                .message_delete(&message.channel, &message.id)
                .await
                .ok();
        }
    }
}

fn content_vec(input: Vec<String>) -> String {
    let mut buffer = String::new();
    input.into_iter().for_each(|mut item| {
        item += " ";
        buffer += &item;
    });
    buffer
}

fn subcommand(input: &Option<String>) -> Vec<String> {
    let mut content: Vec<String> = input
        .clone()
        .unwrap_or_default()
        .split(' ')
        .map(|item| item.to_string())
        .collect();
    content.remove(0);
    content.remove(0);
    content
}

pub fn masq_from_vec(content: Vec<String>) -> Option<MasqueradeData> {
    match content.as_slice() {
        [name, avatar, color] => Some(MasqueradeData {
            name: name.to_string(),
            avatar: Some(avatar.to_owned()),
            color: Some(color.to_owned()),
            aliases: None,
        }),

        [name, avatar] => Some(MasqueradeData {
            name: name.to_string(),
            avatar: Some(avatar.to_owned()),
            ..Default::default()
        }),

        [name] => Some(MasqueradeData {
            name: name.to_string(),
            ..Default::default()
        }),

        _ => None,
    }
}

async fn profiles_fn(client: &Client, message: &Message) {
    // assumed content remove 2

    let mut content = subcommand(&message.content);

    let db = DB::collection::<PluralUser>(COL).await.unwrap();

    let set_content: Option<String> = match content.index(0).as_str() {
        "create" => {
            content.remove(0);

            let profile: Option<MasqueradeData> = masq_from_vec(content);
            let Some(profile) = profile else {
                client
                    .message_send(
                        &message.channel,
                        &DataMessageSend::new().set_content("**invalid profile!**"),
                    )
                    .await
                    .unwrap();
                return;
            };

            if user_find(&db, &message.author).await.unwrap().is_none() {
                let new_user = PluralUser {
                    user_id: message.author.clone(),
                    ..Default::default()
                }
                .insert_profile(&profile);

                db.insert_one(new_user, None).await.unwrap();
            };

            Some("**Profile added!**".to_owned())
        }
        "update" => {
            content.remove(0);

            let profile = masq_from_vec(content);

            let Some(partial_profile) = profile else {
                client
                    .message_send(
                        &message.channel,
                        &DataMessageSend::new().set_content("**invalid profile!**"),
                    )
                    .await
                    .unwrap();
                return;
            };

            if let Some(mut user) = user_find(&db, &message.author).await.unwrap() {
                // println!("{}", serde_json::to_string_pretty(&user).unwrap());
                let Some(old_profile) = user.profiles.get(&partial_profile.name) else {
                    client
                        .message_send(
                            &message.channel,
                            &DataMessageSend::new().set_content("**invalid profile!**"),
                        )
                        .await
                        .unwrap();
                    return;
                };

                println!("{}", serde_json::to_string_pretty(&user).unwrap());

                let new_profile = old_profile.clone().data_override(partial_profile);
                user.profiles.remove(&new_profile.name);
                user.profiles.insert(new_profile.clone().name, new_profile);

                db.delete_one(doc! {"user_id": message.author.clone()}, None)
                    .await
                    .unwrap();

                db.insert_one(user, None).await.unwrap();
            }

            Some(String::from("completed uwu"))
        }
        "search" => match profile_find(&db, &message.author, &content[1]).await {
            Ok(Some(a)) => {
                let a: Masquerade = a.into();

                Some(format!(
                    "```json\n{}",
                    serde_json::to_string_pretty(&a).unwrap()
                ))
            }
            Ok(None) => Some("**couldn't find profile pwp**".to_string()),
            Err(_) => Some("**DB error! <@01FSRTTGJC1XJ6ZEQJMSX8Q96C>**".to_string()),
        },
        "remove" => {
            content.remove(0);

            if let Some(mut user) = user_find(&db, &message.author).await.unwrap() {
                let Some(_) = user.profiles.get(&content[0]) else {
                    client
                        .message_send(
                            &message.channel,
                            &DataMessageSend::new().set_content("**invalid profile!**"),
                        )
                        .await
                        .unwrap();
                    return;
                };

                user.profiles.remove(&content[0]);

                //
                // db.delete_one(doc! {"user_id": message.author.clone()}, None)
                //     .await
                //     .unwrap();

                // db.insert_one(user, None).await.unwrap();

                let filter = doc! {};
                let update = doc! {
                   
                   "$set": {"profiles": {}}
                };

                db.update_one(filter, update, None).await.unwrap();
           

                /*
                                {
                  "user_id": "01FSRTTGJC1XJ6ZEQJMSX8Q96C",
                  "profiles": {
                    "toast": {
                      "name": "toast",
                      "avatar": "https://autumn.revolt.chat/avatars/jlpjowtvnpbgtekkpcv770b4zhy9dqonugqmf543nq/kaias_nice_pfpa.jpg",
                      "color": null,
                      "aliases": null
                    }
                  }
                } */

                /*
                                // Update the document:
                let update_result = movies.update_one(
                   doc! {
                      "_id": &movie.get("_id")
                   },
                   doc! {
                      "$set": { "year": 2019 }
                   },
                   None,
                ).await?;
                println!("Updated {} document", update_result.modified_count); */

                //
            }

            Some(String::from("completed uwu"))
        }

        "send" => {
            content.remove(0);
            let profile_name = content[0].clone();
            content.remove(0);

            let message_content: String = content_vec(content);

            let Ok(Some(profile)) = profile_find(&db, &message.author, &profile_name).await else {
                client
                    .message_send(
                        &message.channel,
                        &DataMessageSend::new().set_content("**invalid profile!**"),
                    )
                    .await
                    .unwrap();
                return;
            };

            let masq: Masquerade = profile.into();

            client
                .message_delete(&message.channel, &message.id)
                .await
                .ok();

            let data = DataMessageSend::new()
                .set_content(&message_content)
                .set_masquerade(&masq);
            client.message_send(&message.channel, &data).await.ok();

            None
        }
        _ => Some(HELP.to_owned()),
    };

    if let Some(content) = set_content {
        client
            .message_send(
                &message.channel,
                &DataMessageSend::new().set_content(&content),
            )
            .await
            .unwrap();
    }
}

async fn alias(_client: &Client, message: &Message) {
    // ?p alias create <profile> <alias>
    // ?p alias delete <profile> <alias>
    // ?p alias purge <profile>
    let content = subcommand(&message.content);

    match content[0].as_str() {
        "create" => {}
        "delete" => {}
        "purge" => {}
        _ => {}
    };
}

async fn user_find(
    db: &Collection<PluralUser>,
    user_id: &str,
) -> Result<Option<PluralUser>, mongodb::error::Error> {
    db.find_one(doc!("user_id": user_id), None).await
}

async fn profile_find(
    db: &Collection<PluralUser>,
    user_id: &str,
    profile_name: &str,
) -> Result<Option<MasqueradeData>, mongodb::error::Error> {
    Ok(match user_find(db, user_id).await? {
        Some(item) => item.profiles.get(profile_name).cloned(),
        None => None,
    })
}
