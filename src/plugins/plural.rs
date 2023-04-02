//use easymongo::mongo::Mongo;
// external

use bson::doc;
use easymongo::mongo::Mongo;
use mongodb::Collection;
use reywen::{
    client::Do,
    structs::message::{DataMessageSend, Masquerade, Reply},
};
use serde::{Deserialize, Serialize};

use crate::common::{crash_condition, md_fmt, RE};

// config struct
// this optional struct adds configurable paramaters that are hot changeable, config files are
// jsons and usually stored in config/
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Plural {
    pub enabled: bool,
    pub channel_only: bool,
    pub channel: String,
    pub collection: String,
}

// plugin main is responsible for getting details and activiating functions based on conditions
pub async fn plural_main(client: &Do) {
    let help = format!(
        "### Plural\n{} {}\n{} {}\n{} {}\n{} {}",
        md_fmt("search", RE::Search),
        "searches for a profile",
        md_fmt("query", RE::Json),
        "Search but returns a JSON",
        md_fmt("rm", RE::Rm),
        "removes a profile",
        md_fmt("insert", RE::Insert),
        "Created a new profile",
    );

    // import plural
    let plural: Plural = serde_json::from_str(
        &(String::from_utf8(
            std::fs::read("config/plural.json").expect("failed to read config/plural.json\n{e}"),
        )
        .unwrap()),
    )
    .expect("Failed to deser plural.json");

    // import mongo

    let mongo: Mongo = serde_json::from_str(
        &(String::from_utf8(
            std::fs::read("config/mongo.json").expect("failed to read config/mongo.json\n{e}"),
        )
        .unwrap()),
    )
    .expect("Failed to deser plural.json");

    // if the config channel matches the channel of the message received AND
    // if the plugin is enabled, send ID
    if !plural.enabled {
        return;
    };

    if plural.channel_only && plural.channel != client.input().channel() {
        return;
    };

    let convec = client.input().convec();

    if crash_condition(&client.input_message, Some("?p")) {
        return;
    };

    // additional crash condition
    if convec.len() < 3 {
        client.message().sender(&help).await;
        return;
    };

    if convec[1] == *"help" {
        let res = match convec[2].as_str() {
            "search" => "**Search**\n`?p search <profilename>`",
            "query" => "**Query**\n`?p query <profilename>`",
            "rm" => "**Remove**\n`?p rm <profilename>`",
            "insert" => "**Insert**\n`?p insert <profilename>`\n\nOptionals\n`--avatar <url>`\n`--colour <colour>`",
            _ => &help,
        };

        client.message().sender(res).await;
    }

    let database = Mongo::new()
        .username(&mongo.username)
        .password(&mongo.password)
        .database(&mongo.database)
        .db_generate()
        .await;

    let db = database.collection::<Masquerade>(&plural.collection);

    match &convec[1] as &str {
        "search" => cli_search(client.clone(), db).await,

        "rm" => pl_remove(client.clone(), db).await,

        "insert" => pl_insert(client, db).await,

        "send" => pl_send(client.clone(), db).await,

        "query" => pl_query(db, client).await,
        _ => {}
    }
}

async fn cli_search(client: Do, db: Collection<Masquerade>) {
    let res = match pl_search(&client.input().convec()[2], db).await {
        Some(_) => "**Profile found!**",
        None => "**Profile could not be found!**",
    };
    client.message().sender(res).await;
}

async fn pl_search(query: &str, db: Collection<Masquerade>) -> Option<Masquerade> {
    db.find_one(doc! { "name": query }, None).await.unwrap()
}

async fn pl_remove(client: Do, collection: Collection<Masquerade>) {
    let convec = client.input().convec();

    let userquery = collection.find_one(doc! { "name": &convec[2] }, None).await;

    let res = match userquery {
        Err(_) => "**Failed to connect to mongodb**",
        Ok(None) => "**No results found!**",
        Ok(Some(_)) => "DEL",
    };

    if res != "DEL" {
        client.message().sender(res).await;
        return;
    };

    let del_res = collection.delete_one(doc! {"name": &convec[2]}, None).await;

    let str = match del_res {
        Ok(_) => String::from("**Successfully deleted**"),
        Err(e) => format!("**Error**\n```text\n{e}"),
    };
    client.message().sender(&str).await;
}

async fn pl_send(client: Do, db: Collection<Masquerade>) {
    let convec = client.input().convec();

    // ?p send <>
    let profile = match pl_search(&convec[2], db).await {
        Some(a) => a,
        None => {
            client
                .message()
                .sender("**Invalid profile! we couldn't find it pwp**")
                .await;
            return;
        }
    };

    // turn the query into a sendable string
    let mut message = convec;
    message.remove(0);
    message.remove(0);
    message.remove(0);
    let new_message: String = message.iter().map(|i| i.to_string() + " ").collect();

    let mut replies_payload = Vec::new();
    if let Some(replies) = client.input().replies() {
        for x in replies {
            let reply = Reply::new().id(&x);
            replies_payload.push(reply);
        }
    }
    let payload = DataMessageSend::new()
        .masquerade(profile)
        .content(&new_message)
        .replies(replies_payload);

    let message = client.message();

    tokio::join!(message.send(payload), message.delete(None));
}

async fn pl_insert(client: &Do, db: Collection<Masquerade>) {
    let collection = db.clone();

    let convec = client.input().convec();

    if pl_search(&convec[2], db).await.is_some() {
        client
            .message()
            .sender("**This profile already exists! try another name**")
            .await;
        return;
    };

    // CLI schema out of order ?p insert FLoofy --colour red --avatar img.jpg
    // no matter what there is always name

    let mut masq = Masquerade::new().name(&convec[2]);

    // validity check and optional insertion
    /*
    for x in 0..convec.len() - 1 {
        match convec[x].as_str() {
            "--avatar" | "-a" => {
                if convec[x + 1].chars().count() < 256 {
                    masq = masq.avatar(&convec[x + 1]);
                };
            }
            "--color" | "--colour" | "-c" => {
                if convec[x + 1].chars().count() < 128 {
                    masq = masq.colour(&convec[x + 1]);
                };
            }

            _ => {}
        }
    }

    */

    for x in 0..convec.len() - 1 {
        let charlen = convec[x + 1].chars().count();

        match (convec[x].clone().as_str(), charlen) {
            ("--color" | "--colour" | "-c", size) => {
                if size > 128 {
                    client
                        .message()
                        .sender("**Invalid character limit for colour field**")
                        .await;
                    return;
                };
                masq = masq.colour(&convec[x + 1]);
            }

            ("--avatar" | "-a", size) => {
                if size > 256 {
                    client
                        .message()
                        .sender("**Invalid character limit for avatar field**")
                        .await;
                    return;
                };
                masq = masq.colour(&convec[x + 1]);
            }
            _ => {}
        }
    }

    let res = match collection.insert_one(masq, None).await {
        Ok(_) => "**Valid profile! adding to collection**",
        Err(_) => "**Failed to connect**",
    };
    client.message().sender(res).await;
}

async fn pl_query(db: Collection<Masquerade>, client: &Do) {
    let userquery = match pl_search(&client.input().convec()[2], db).await {
        Some(a) => a,
        None => {
            client.message().sender("**Could not find profile!**").await;
            return;
        }
    };

    let mes = serde_json::to_string_pretty(&userquery).unwrap();

    client
        .message()
        .sender(&format!("```json\n{mes}\n```"))
        .await;
}
