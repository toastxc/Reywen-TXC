use bson::doc;
use futures_util::TryStreamExt;
use mongodb::Collection;
use reywen::{
    delta::{
        fs::fs_to_str,
        lreywen::{convec, crash_condition},
        mongo::mongo_db,
        oop::Reywen,
    },
    quark::{delta::message::RMessage, mongo::RMongo},
};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{md_fmt, RE};

#[derive(Deserialize, Serialize)]
pub struct TomoConf {
    pub enabled: bool,
    pub collection: String,
}

pub async fn t_main(client: &Reywen, input_message: &RMessage) {
    let help = format!(
        "### Tomo\n{} {}\n{} {}\n {} {}\n {} {}\n{} {}",
        md_fmt("enrol", RE::Json),
        "Registers self to the game",
        md_fmt("exit", RE::Rm),
        "Removes self from the game",
        md_fmt("check", RE::Search),
        "Displays user profile",
        md_fmt("buy", RE::Insert),
        "Attempt to purchase animal",
        md_fmt("dev", RE::Json),
        "dev commands - sudoers only",
    );

    let client: Reywen = client.to_owned();
    // cli stuff
    if crash_condition(input_message, Some("?t")) {
        return;
    };
    let convec = reywen::delta::lreywen::convec(input_message);

    if convec[1] == "help" {
        client.sender(&help).await;
        return;
    };

    // connect to and import database

    let db_str = fs_to_str("config/mongo.json").unwrap();
    let db_conf: RMongo = serde_json::from_str(&db_str).unwrap();

    let tomo_str = fs_to_str("config/tomo.json").unwrap();
    let tomo_conf: TomoConf = serde_json::from_str(&tomo_str).unwrap();

    if !tomo_conf.enabled {
        return;
    };

    let db = mongo_db(
        RMongo::new()
            .username(&db_conf.username)
            .password(&db_conf.password)
            .database(&db_conf.database),
    )
    .await
    .collection::<TProfile>(&tomo_conf.collection);

    match convec[1] {
        "enrol" => add_self(client, input_message, db).await,
        "exit" => remove_self(client, db).await,
        "check" => query_self(client, input_message, db).await,
        "dev" => dev_patterns(client, db).await,
        "buy" => buy_pet(client, db).await,
        _ => {
            client.sender("**Invalid command**").await;
        }
    };
}

async fn buy_pet(client: Reywen, db: Collection<TProfile>) {
    if !user_exist(&db, &client.input_message.author).await {
        client.sender("**User does not exist!**").await;
        return;
    };

    let user = db
        .find_one(doc!("user_id": &client.input_message.author), None)
        .await
        .unwrap()
        .unwrap();

    let convec = convec(&client.input_message);

    // ?t buy frog

    let animal = match Animal::to_enum(convec[2]) {
        None => {
            client.sender("**invalid animal**").await;
            return;
        }
        Some(a) => a,
    };

    if user.money < Animal::cost(&animal) {
        client.sender("**Cannot buy, not enough coins**").await;
        return;
    };

    let mut animal_vec = vec![Animal::to_str(&animal)];

    for x in user.animals.iter() {
        animal_vec.push(Animal::to_str(x))
    }

    let newmoney = user.money - Animal::cost(&animal);

    let updater = vec![
        doc! {"$set": {"money": newmoney}},
        doc! {"$set": {"animals": animal_vec}},
    ];
    db.update_one(
        doc! {"user_id": &client.input_message.author},
        updater,
        None,
    )
    .await
    .unwrap();
}

async fn remove_self(client: Reywen, db: Collection<TProfile>) {
    if !user_exist(&db, &client.input_message.author).await {
        client.sender("**User does not exist!**").await;
        return;
    };

    db.delete_one(doc!("user_id": &client.input_message.author), None)
        .await
        .unwrap();

    client.sender("**user removed**").await;
}

async fn dev_patterns(client: Reywen, db: Collection<TProfile>) {
    if !client.auth.sudoers.contains(&client.input_message.author) {
        client
            .sender("**You are not authorised to use dev commands**")
            .await;
        return;
    };
    // ?t dev newday

    match convec(&client.input_message)[2] {
        "newday" => newday(client, db).await,
        _ => {
            client.sender("**Invalid command**").await;
        }
    }
}

async fn newday(client: Reywen, db: Collection<TProfile>) {
    // iterator for document stream
    let cursor = db.find(None, None).await.unwrap();

    // define database locally
    let database: Vec<TProfile> = cursor.try_collect().await.unwrap();

    // for every item in db, update money
    for x in database.iter() {
        let updater = doc! {"$set": {"money":x.money + coin_calc(x)}};

        db.update_one(doc! {"user_id": &x.user_id}, updater, None)
            .await
            .unwrap();
    }
    client.sender(":cat_sussy:  :thumbsup: ").await;
}

fn coin_calc(profile: &TProfile) -> u32 {
    let mut fin = 0;

    for x in profile.animals.iter() {
        fin += Animal::value(x);
    }
    fin
}
async fn query_self(client: Reywen, input_message: &RMessage, db: Collection<TProfile>) {
    if !user_exist(&db, &input_message.author).await {
        client.sender("**User doesn't exist!**").await;
        return;
    };

    let res = match db
        .find_one(doc!("user_id": &input_message.author), None)
        .await
    {
        Ok(a) => format!(
            "```json\n\n{}\n```\n",
            serde_json::to_string_pretty(&a).unwrap()
        ),
        Err(_) => String::from("**Could not find user!**"),
    };

    client.sender(&res).await;
}

async fn add_self(client: Reywen, input_message: &RMessage, db: Collection<TProfile>) {
    // ?t add_self
    if user_exist(&db, &input_message.author).await {
        client.sender("**User already exists**").await;
        return;
    };
    let profile = TProfile::new(&input_message.author);

    let res = match db.insert_one(profile, None).await {
        Err(e) => format!("**Failed inserting user!**\n{e}"),

        Ok(_) => format!("**Successfully added user <@{}>**", input_message.author),
    };

    client.sender(&res).await;
}

async fn user_exist(db: &Collection<TProfile>, target: &str) -> bool {
    db.find_one(doc!("user_id": target), None)
        .await
        .unwrap()
        .is_some()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TProfile {
    pub user_id: String,
    pub animals: Vec<Animal>,
    pub money: u32,
}
impl TProfile {
    fn new(user: &str) -> Self {
        TProfile {
            money: 20,
            user_id: String::from(user),
            ..Default::default()
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum Animal {
    #[default]
    Void,
    Penguin,
    Dog,
    Cat,
    Fish,
    Dragon,
    Hyena,
}

impl Animal {
    pub fn value(&self) -> u32 {
        match self {
            Animal::Void => 0,
            Animal::Penguin => 1,
            Animal::Dog => 2,
            Animal::Cat => 3,
            Animal::Fish => 4,
            Animal::Dragon => 5,
            Animal::Hyena => 6,
        }
    }
    pub fn to_enum(input: &str) -> Option<Self> {
        match input {
            "penguin" | "Penguin" => Some(Animal::Penguin),
            "dog" | "Dog" => Some(Animal::Dog),
            "cat" | "Cat" => Some(Animal::Cat),
            "fish" | "Fish" => Some(Animal::Fish),
            "Dragon" | "dragon" => Some(Animal::Dragon),
            "hyena" | "Hyena" => Some(Animal::Hyena),

            _ => None,
        }
    }
    pub fn cost(&self) -> u32 {
        match self {
            Animal::Void => 1000000000,
            Animal::Penguin => 10,
            Animal::Dog => 20,
            Animal::Cat => 30,
            Animal::Fish => 40,
            Animal::Dragon => 50,
            Animal::Hyena => 60,
        }
    }

    fn to_str(x: &Animal) -> String {
        match x {
            Animal::Void => "Void",
            Animal::Penguin => "Penguin",
            Animal::Dog => "Dog",
            Animal::Cat => "Cat",
            Animal::Fish => "Fish",
            Animal::Dragon => "Dragon",
            Animal::Hyena => "Hyena",
        }
        .to_string()
    }
}
