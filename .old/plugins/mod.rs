use std::collections::HashMap;

use reywen::structures::channels::message::Message;
use serde::{Deserialize, Serialize};

pub mod federolt;
pub mod pluralkit;
pub fn conf_from_file_toml<T: serde::de::DeserializeOwned>(path: &str) -> T {
    toml::from_str::<T>(&String::from_utf8(std::fs::read(path).unwrap()).unwrap()).unwrap()
}
pub fn conf_from_file_json<T: serde::de::DeserializeOwned>(path: &str) -> T {
    serde_json::from_str::<T>(&String::from_utf8(std::fs::read(path).unwrap()).unwrap()).unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    pub token: String,
    pub bot_id: String,
    pub sudoers: Option<HashMap<u32, Vec<String>>>,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct StartInfo<T: Default + Clone> {
    pub enabled: bool,
    pub plugin_name: String,
    pub ignore_self: bool,
    pub requires_sudo: Option<u32>,
    pub data: T,
}

impl<T: Clone + Default> StartInfo<T> {
    pub fn from_message_input(&self, message: &Message, auth: &Auth) -> Option<T> {
        if !Self::sudo_check(auth, self.requires_sudo, &message.author) {
            return None;
        };
        if !self.enabled {
            return None;
        };

        if self.ignore_self && message.author == auth.bot_id {
            return None;
        };

        Some(self.data.to_owned())
    }

    pub fn enabled(&self) -> Option<T> {
        if self.enabled {
            Some(self.data.to_owned())
        } else {
            None
        }
    }

    // true: access granted
    // false: access denied
    pub fn sudo_check(auth: &Auth, requires_sudo: Option<u32>, user_id: &str) -> bool {
        if requires_sudo.is_none() {
            return true;
        };
        if let Some(required_sudo_group) = requires_sudo {
            if let Some(sudoer_groups) = auth.clone().sudoers {
                if let Some(sudoers) = sudoer_groups.get(&required_sudo_group) {
                    return sudoers.contains(&String::from(user_id));
                }
            }
        }
        false
    }
}
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Start {
    pub pluralkit: StartInfo<()>,
    pub federolt: StartInfo<FederoltGroups>,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct FederoltGroups {
    groups: HashMap<String, Vec<String>>,
}
