// external
use mongodb::{bson::doc, Collection};
use serde::{Deserialize, Serialize};

// internal
use reywen::{
    delta::{
        fs::fs_to_str,
        lreywen::{convec, crash_condition},
        mongo::mongo_db,
        oop::Reywen,
    },
    quark::{
        delta::message::{Masquerade, RMessage, RMessagePayload},
        mongo::RMongo,
    },
};

use crate::{md_fmt, RE};

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
pub async fn plural_main(client: &Reywen, input_message: &RMessage) {
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

    let client: Reywen = client.to_owned();

    // import plural
    let conf = fs_to_str("config/plural.json").expect("failed to read config/plural.json\n{e}");
    let plural: Plural = serde_json::from_str(&conf).expect("Failed to deser plural.json");

    // import mongo
    let mongo_str = fs_to_str("config/mongo.json").unwrap();
    let mongo: RMongo = serde_json::from_str(&mongo_str).expect("Failed to deser plural.json");

    // if the config channel matches the channel of the message received AND
    // if the plugin is enabled, send ID
    if !plural.enabled {
        return;
    };
    if plural.channel_only && plural.channel != input_message.channel {
        return;
    };

    let convec = convec(input_message);

    if crash_condition(input_message, Some("?p")) {
        return;
    };

    // additional crash condition
    if convec.len() < 3 {
        client.sender(&help).await;
        return;
    };

    let dbinfo = RMongo::new()
        .username(&mongo.username)
        .password(&mongo.password)
        .database(&mongo.database);

    let db = mongo_db(dbinfo)
        .await
        .collection::<Masquerade>(&plural.collection);

    match convec[1] as &str {
        "search" => cli_search(client.clone(), db).await,

        "rm" => pl_remove(client.clone(), db).await,

        "insert" => pl_insert(client.clone(), db).await,

        "send" => pl_send(client.clone(), db).await,

        "query" => pl_query(db, client).await,
        _ => {}
    }
}

async fn cli_search(client: Reywen, db: Collection<Masquerade>) {
    if pl_search(convec(&client.input_message)[2], db)
        .await
        .is_none()
    {
        client.sender("**Profile could not be found!**").await;
    } else {
        client.sender("**Profile found!**").await;
    }
}

async fn pl_search(query: &str, db: Collection<Masquerade>) -> Option<Masquerade> {
    db.find_one(doc! { "name": query }, None).await.unwrap()
}

async fn pl_remove(client: Reywen, db: Collection<Masquerade>) {
    let convec = convec(&client.input_message);

    let collection = db;

    let userquery = collection.find_one(doc! { "name": convec[2] }, None).await;

    if userquery.is_err() {
        client.sender("**Failed to connect to mongodb**").await;
    } else if userquery.unwrap().is_none() {
        client.sender("**No results found!**").await;
    } else {
        let del_res = collection.delete_one(doc! {"name": convec[2]}, None).await;
        client
            .clone()
            .sender("**Profile found, deleting...**")
            .await;

        let str = match del_res {
            Ok(_) => String::from("**Successfully deleted**"),
            Err(e) => format!("**Error**\n```text\n{e}"),
        };
        client.sender(&str).await;
    };
}

async fn pl_send(client: Reywen, db: Collection<Masquerade>) {
    let convec: Vec<&str> = convec(&client.input_message);

    // ?p send <>
    let profile = pl_search(convec[2], db).await;

    if profile.is_none() {
        client
            .sender("**Invalid profile! we couldn't find it pwp**")
            .await;
        return;
    };
    let profile = profile.unwrap();

    // turn the query into a sendable string
    let mut message = convec;
    message.remove(0);
    message.remove(0);
    message.remove(0);
    let new_message: String = message.iter().map(|i| i.to_string() + " ").collect();

    let mut payload = RMessagePayload::new()
        .masquerade(profile)
        .content(&new_message);

    // optional fields
    if client.input_message.replies.is_some() {
        payload = payload.reply_from(&client.input_message);
    };

    tokio::join!(
        client.clone().send(payload),
        client.clone().delete_msg(&client.input_message._id),
    );
}

async fn pl_insert(client: Reywen, db: Collection<Masquerade>) {
    let collection = db.clone();

    let convec = convec(&client.input_message);

    if pl_search(convec[2], db).await.is_some() {
        client
            .clone()
            .sender("**This profile already exists! try another name**")
            .await;
        return;
    };

    // CLI schema out of order ?p insert FLoofy --colour red --avatar img.jpg
    // no matter what there is always name

    let mut masq = Masquerade::new().name(convec[2]);

    // validity check and optional insertion
    for x in 0..convec.len() - 1 {
        // colour
        if convec[x] == "--colour" && convec[x + 1].chars().count() < 10 {
            masq = masq.colour(convec[x + 1]);
        };
        // avatar
        if convec[x] == "--avatar" && convec[x + 1].chars().count() < 100 {
            masq = masq.avatar(convec[x + 1]);
        };
    }

    let userquery = collection.insert_one(masq, None).await;

    if userquery.is_err() {
        client.sender("**Failed to connect**").await;
    } else {
        client
            .sender("**Valid profile! adding to collection**")
            .await;
    };
}

async fn pl_query(db: Collection<Masquerade>, client: Reywen) {
    // ?p query somethign
    let userquery = pl_search(convec(&client.input_message)[2], db).await;

    if userquery.is_none() {
        client.sender("**Could not find profile!**").await;
        return;
    };
    let userquery = userquery.unwrap();

    let mut str = format!("```json\n{{\n\"name\": \"{}\"", userquery.name.unwrap());

    if userquery.avatar.is_some() {
        str += &format!(",\n\"avatar\": \"{}\"", userquery.avatar.unwrap());
    };
    if userquery.colour.is_some() {
        str += &format!(",\n\"colour\": \"{}\"", userquery.colour.unwrap());
    };

    str += "\n}\n```\n";
    client.sender(&str).await;
}
