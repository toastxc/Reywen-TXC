// external
use reqwest::header::USER_AGENT;
use reywen::client::Do;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::common::{crash_condition, lte, md_fmt, RE};

// internal

const DURL: &str =
    "https://autumn.revolt.chat/attachments/6bfy1Es-xWa9U6VzEPSw7DnbQPGUDK7LWrk4yRWHpV";
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct E6Conf {
    pub enabled: bool,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Poster {
    #[serde(skip_serializing_if = "Option::is_none", rename = "posts")]
    post: Option<Vec<Post>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub created_at: String,
    pub updated_at: String,
    pub file: File,
    pub preview: Preview,
    pub sample: Sample,
    pub score: Score,
    pub tags: Tags,
    pub locked_tags: Vec<Value>,
    pub change_seq: i64,
    pub flags: Flags,
    pub rating: String,
    pub fav_count: i64,
    pub sources: Vec<String>,
    pub pools: Vec<Value>,
    pub relationships: Relationships,
    pub approver_id: Option<i64>,
    pub uploader_id: Option<i64>,
    pub description: String,
    pub comment_count: i64,
    pub is_favorited: bool,
    pub has_notes: bool,
    pub duration: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub width: i64,
    pub height: i64,
    pub ext: String,
    pub size: i64,
    pub md5: String,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preview {
    pub width: i64,
    pub height: i64,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub has: bool,
    pub height: i64,
    pub width: i64,
    pub url: Option<String>,
    pub alternates: Alternates,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternates {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub up: i64,
    pub down: i64,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    pub general: Option<Vec<String>>,
    pub species: Option<Vec<String>>,
    pub character: Option<Vec<String>>,
    pub copyright: Option<Vec<String>>,
    pub artist: Option<Vec<String>>,
    pub invalid: Option<Vec<String>>,
    pub lore: Option<Vec<String>>,
    pub meta: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flags {
    pub pending: bool,
    pub flagged: bool,
    pub note_locked: bool,
    pub status_locked: bool,
    pub rating_locked: bool,
    pub comment_disabled: bool,
    pub deleted: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationships {
    pub parent_id: Option<i64>,
    pub has_children: bool,
    pub has_active_children: bool,
    pub children: Option<Vec<i32>>,
}

pub async fn e6_main(client: &Do) {
    let help = format!(
        "### E621\n{} {}",
        md_fmt("search", RE::Search),
        "searches for post",
    );

    // import config
    let e6: E6Conf = serde_json::from_str(
        &String::from_utf8(
            std::fs::read("config/e6.json").expect("failed to read config/e6.json\n{e}"),
        )
        .unwrap(),
    )
    .expect("invalid config");

    let convec = client.input().convec();

    if crash_condition(&client.input_message, Some("?e")) {
        return;
    };

    if convec[0] == "help" {}

    let var = match &convec[1] as &str {
        "search" => e6_search(&convec, &e6.url).await,
        "help" => Some(help),
        _ => return,
    };

    if let Some(message) = var {
        client.message().sender(&message).await;
        return;
    };

    // on failure
    if ping_test(&e6.url).await {
        client
            .message()
            .sender(&format!("**Could not reach {}", e6.url))
            .await;
    };
}

async fn ping_test(url: &str) -> bool {
    let client: std::result::Result<reqwest::Response, reqwest::Error> =
        reqwest::Client::new().get(url).send().await;

    if client.is_ok() {
        return false;
    };
    true
}

async fn e6_search(convec: &[String], url: &str) -> Option<String> {
    // https://e926.net/posts?tags=fox&limit=1&page=2
    // ?e search fox 2

    // query payload url - tags - page number
    let query = &format!(
        "{url}/posts.json?tags={}&limit=1&page={}",
        encode(&convec[2]),
        numcheck(convec)
    );

    // http request
    let http: std::result::Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
        .get(query)
        // user agent used with permission
        .header(USER_AGENT, "libsixgrid/1.1.1")
        .send()
        .await;

    if http.is_err() {
        return None;
    };

    let http_payload = match http {
        Ok(a) => a.text().await.unwrap(),
        Err(_) => return None,
    };

    if http_payload.is_empty() {
        return None;
    };

    if let Ok(poster) = serde_json::from_str::<Poster>(&http_payload) {
        if poster.post.is_none() {
            return Some(String::from("**No Results!**"));
        };

        if let Some(post) = poster.post {
            if !post.is_empty() {
                return match &post[0].file.url {
                    Some(a) => Some(format!("**UwU**\n{}", lte(a))),
                    None => Some(DURL.to_string()),
                };
            }
        }
    }
    Some(String::from("**Failed to get results!**"))
}

fn numcheck(convec: &[String]) -> String {
    if convec.len() < 4 {
        return 1.to_string();
    };

    let maybe_number = convec[3].parse::<usize>();

    let number = maybe_number.unwrap_or(1);

    let res = if number >= 750 { 1 } else { number };

    res.to_string()
}
