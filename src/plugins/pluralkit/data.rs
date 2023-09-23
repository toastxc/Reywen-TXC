use bson::{doc, Document};
use indexmap::IndexSet;
use reywen::structures::channels::message::Masquerade;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Profile {
    #[serde(rename = "_id")]
    pub id: Composite,
    pub data: Masquerade,
    pub aliases: Option<IndexSet<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProfilePrint {
    data: Masquerade,
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<IndexSet<String>>,
    id: u32,
}

impl Profile {
    pub fn alias(&self, alias: &str) -> bool {
        self.aliases
            .as_ref()
            .is_some_and(|aliases| aliases.contains(alias))
    }
    pub fn format(&self) -> String {
        serde_json::to_string_pretty(
            &(ProfilePrint {
                data: self.data.clone(),
                aliases: self.aliases.clone(),
                id: self.id.profile_id,
            }),
        )
        .unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Composite {
    pub user: String,
    #[serde(rename = "profile")]
    pub profile_id: u32,
}

pub fn arg(name: Option<&String>, avatar: Option<&String>, color: Option<&String>) -> Document {
    let mut document = Document::new();

    if let Some(update) = bson_parse(name, "name") {
        document.extend(update);
    }
    if let Some(update) = bson_parse(avatar, "avatar") {
        document.extend(update);
    }
    if let Some(update) = bson_parse(color, "color") {
        document.extend(update);
    }

    let doc_wrap = doc! {"$set": document};
    doc_wrap
}

fn bson_parse(input: Option<&String>, name: &str) -> Option<Document> {
    let input_data = input?.clone();
    let input_check = input_data.to_lowercase();

    match input_check.as_str() {
        "skip" | "next" => None,
        "none" | "null" => Some(doc! {format!("data.{name}"): null}),
        _ => Some(doc! {format!("data.{name}"): input_data}),
    }
}

pub enum Multi<T> {
    Many(Vec<T>),
    Single(Option<T>),
}

pub fn masq_from_vec(content: Vec<String>) -> Option<Masquerade> {
    match content.as_slice() {
        [name, avatar, color] => Some(Masquerade {
            name: Some(name.to_string()),
            avatar: Some(avatar.to_owned()),
            colour: Some(color.to_owned()),
        }),

        [name, avatar] => Some(Masquerade {
            name: Some(name.to_string()),
            avatar: Some(avatar.to_owned()),
            ..Default::default()
        }),

        [name] => Some(Masquerade {
            name: Some(name.to_string()),
            ..Default::default()
        }),

        _ => None,
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ProfileAlias {
    pub alias: String,
    #[serde(rename = "_id")]
    pub id: Composite,
}

impl ProfileAlias {
    pub fn new(id: Composite) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
    pub fn burn(&mut self, alias: impl Into<String>) -> Self {
        self.alias = alias.into();
        self.to_owned()
    }
}

pub fn message_vec(input: impl Into<Vec<String>>) -> String {
    input
        .into()
        .into_iter()
        .map(|bit| format!(" {bit}"))
        .collect()
}
