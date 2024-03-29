use super::{data::Multi, DB_ERROR};
use crate::plugins::pluralkit::data::{message_vec, ProfileAlias};
use crate::plugins::pluralkit::db::profile_format;
use crate::plugins::pluralkit::{help, HELP_ALIASES, HELP_PROFILES, MULTIPLE_NAME};
use crate::{
    plugins::pluralkit::{
        data::{arg, masq_from_vec, Composite, Profile},
        NOT_FOUND, SUCCESS_EMPTY,
    },
    DB,
};

use mongodb::results::UpdateResult;
use reywen::{
    client::{methods::message::DataMessageSend, Client},
    structures::channels::message::Message,
};

pub async fn on_message(client: &Client, message: &Message, db: &DB) {
    let Some(mut content) = message.content_contains("?p", " ") else {
        return;
    };

    match content.len() {
        0 | 1 | 2 => {
            client
                .message_send(
                    &message.channel,
                    &DataMessageSend::new().set_content(&help()),
                )
                .await
                .unwrap();
            return;
        }

        _ => content.remove(0),
    };

    match content.first().unwrap().as_str() {
        "alias"  | "aliases" => alias_fn(client, message, content, db).await,
        "profile" | "profiles" => profile_fn(client, message, content, db).await,
        _ => {
            let alias = content.remove(0);

            let smee = db.alias_check_smart(&message.author, alias).await;

            let data: Option<DataMessageSend> = match smee {
                Err(_) => Some(DataMessageSend::new().set_content(DB_ERROR)),
                Ok(Some(profile)) => {
                    println!("gonna_send{:?}", content);
                    Some(
                        DataMessageSend::new()
                            .set_content(&message_vec(content.clone()))
                            .set_masquerade(&profile.data),
                    )
                }
                _ => None,
            };

            if let Some(data) = data {
                tokio::join!(
                    client.message_send(&message.channel, &data),
                    client.message_delete(&message.channel, &message.id)
                );
            }
        }
    }
}

async fn alias_fn(client: &Client, message: &Message, mut content: Vec<String>, db: &DB) {
    content.remove(0);

    println!("{content:?}");
    if content.len() < 2 {
        client
            .message_send(
                &message.channel,
                &DataMessageSend::new().set_content(HELP_ALIASES),
            )
            .await
            .unwrap();
        return;
    };

    // CRUD
    match content.first().unwrap().to_lowercase().as_str() {
        "make" | "create" | "delete" | "remove" => {
            let content = match content.remove(0).as_str() {
                "create" | "make" => match db
                    .plural
                    .profile_find_many_smart(&message.author, content.remove(0))
                    .await
                {
                    Ok(Multi::Many(_)) => MULTIPLE_NAME,
                    Err(_) => DB_ERROR,
                    Ok(Multi::Single(None)) => NOT_FOUND,
                    Ok(Multi::Single(Some(profile))) => {
                        println!("TO CREATE {:?}", content);
                        let profile_alias = ProfileAlias {
                            alias: content.remove(0),
                            id: Composite {
                                user: message.author.clone(),
                                profile_id: profile.id.profile_id,
                            },
                        };
                        match db.aliases.alias_create(profile_alias).await {
                            Ok(_) => SUCCESS_EMPTY,
                            Err(_) => DB_ERROR,
                        }
                    }
                }
                .to_string(),
                "delete" | "remove" => match db
                    .aliases
                    .alias_delete(&message.author, content.first().unwrap())
                    .await
                {
                    Ok(_) => SUCCESS_EMPTY,
                    Err(_) => DB_ERROR,
                }
                .to_string(),
                _ => {
                    panic!("illogical")
                }
            };
            client
                .message_send(
                    &message.channel,
                    &DataMessageSend::new().set_content(&content),
                )
                .await
                .unwrap();

            // ?p alias create ID/NAME alias
        }
        _ => {}
    }
}

pub async fn profile_fn(client: &Client, message: &Message, mut content: Vec<String>, db: &DB) {
    content.remove(0);
    let db = &db.plural;

    let new_content = match content.remove(0).to_lowercase().as_str() {
        "create" | "make" => {
            // ?p profile create <profilename> XX
            println!("PROFILE CLI: {:?}", content);
            let profile = Profile {
                id: Composite {
                    user: message.author.clone(),
                    profile_id: rand::random(),
                },
                data: masq_from_vec(content).unwrap(),
            };
            println!("{}", serde_json::to_string_pretty(&profile).unwrap());
            match db.0.write().await.insert_one(profile, None).await {
                Ok(_) => SUCCESS_EMPTY,
                Err(_) => DB_ERROR,
            }
            .to_string()
        }
        "fetch" | "search" | "get" | "read" => {
            match db
                .profile_find_many_smart(&message.author, &content[0])
                .await
            {
                Ok(Multi::Many(data)) => profile_format(data),
                Ok(Multi::Single(Some(data))) => profile_format(vec![data]),
                Ok(Multi::Single(None)) => NOT_FOUND.to_string(),
                Err(_) => DB_ERROR.to_string(),
            }
        }
        "edit" | "update" => {
            match db
                .profile_find_many_smart(&message.author, &content[0])
                .await
            {
                Ok(Multi::Many(data)) => {
                    format!("{MULTIPLE_NAME}\n\n{}", profile_format(data))
                }
                Ok(Multi::Single(None)) => NOT_FOUND.to_string(),
                Ok(Multi::Single(Some(data))) => match db
                    .profile_edit(
                        &data.id.user,
                        data.id.profile_id,
                        arg(content.get(1), content.get(2), content.get(3)),
                    )
                    .await
                {
                    Ok(UpdateResult {
                        modified_count: 0, ..
                    }) => NOT_FOUND,
                    Ok(UpdateResult {
                        modified_count: _, ..
                    }) => SUCCESS_EMPTY,
                    Err(_) => DB_ERROR,
                }
                .to_string(),
                Err(_) => DB_ERROR.to_string(),
            }
        }
        "delete" | "remove" => {
            match db
                .profile_find_many_smart(&message.author, &content[0])
                .await
            {
                Ok(Multi::Many(data)) => {
                    format!("{MULTIPLE_NAME}\n{}", profile_format(data))
                }
                Ok(Multi::Single(None)) => NOT_FOUND.to_string(),
                Ok(Multi::Single(Some(data))) => {
                    match db.profile_delete(data.id.user, data.id.profile_id).await {
                        Ok(a) => match a.deleted_count {
                            0 => NOT_FOUND.to_string(),
                            _ => SUCCESS_EMPTY.to_string(),
                        },
                        Err(_) => DB_ERROR.to_string(),
                    }
                }
                Err(_) => DB_ERROR.to_string(),
            }
        }
        "send" | "message" | "msg" => {
            match db
                .profile_find_many_smart(&message.author, content.remove(0))
                .await
            {
                Ok(Multi::Many(data)) => format!("{MULTIPLE_NAME}\n{}", profile_format(data)),
                Ok(Multi::Single(Some(data))) => {
                    let data = &DataMessageSend::new()
                        .set_content(&message_vec(content))
                        .set_masquerade(&data.data);

                    tokio::join!(
                        client.message_delete(&message.channel, &message.id),
                        client.message_send(&message.channel, data)
                    );
                    return;
                }
                Ok(Multi::Single(None)) => NOT_FOUND.to_string(),
                Err(_) => DB_ERROR.to_string(),
            }
        }
        _ => HELP_PROFILES.to_string(),
    };
    client
        .message_send(
            &message.channel,
            &DataMessageSend::new().set_content(&new_content),
        )
        .await
        .unwrap();
}
