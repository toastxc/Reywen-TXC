use bson::doc;
use easymongo::mongo::Mongo;
use futures_util::TryStreamExt;
use mongodb::Collection;

use reywen::client::Do;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{crash_condition, md_fmt, RE};

#[derive(Deserialize, Serialize)]
pub struct TomoConf {
    pub enabled: bool,
    pub collection: String,
}

pub async fn t_main(client: &Do) {
    let help = format!(
        "### Tomo\n{} {}\n{} {}\n {} {}\n {} {}\n{} {}\n{} {}",
        md_fmt("enrol", RE::Json),
        "Registers self to the game",
        md_fmt("exit", RE::Rm),
        "Removes self from the game",
        md_fmt("check", RE::Search),
        "Displays user profile",
        md_fmt("buy", RE::Insert),
        "Attempt to purchase animal",
        md_fmt("shop", RE::Insert),
        "Lists available animals and prices",
        md_fmt("dev", RE::Json),
        "dev commands - sudoers only",
    );

    // cli stuff
    if crash_condition(&client.input_message, Some("?t")) {
        return;
    };
    let convec = client.input().convec();

    if convec[1] == "help" {
        client.message().sender(&help).await;
        return;
    }

    // connect to and import database

    let db_conf: Mongo = serde_json::from_str(
        &(String::from_utf8(
            std::fs::read("config/mongo.json").expect("failed to read config/mongo.json\n{e}"),
        )
        .unwrap()),
    )
    .expect("Failed to deser plural.json");

    let tomo_conf: TomoConf = serde_json::from_str(
        &String::from_utf8(
            std::fs::read("config/tomo.json").expect("failed to read config/tomo.json\n{e}"),
        )
        .unwrap(),
    )
    .expect("Failed to deser plural.json");

    if !tomo_conf.enabled {
        return;
    };

    let db = Mongo::new()
        .username(&db_conf.username)
        .password(&db_conf.password)
        .database(&db_conf.database)
        .db_generate()
        .await
        .collection::<TProfile>(&tomo_conf.collection);

    let shop = "
    Void: `Do not attempt to purchase`
    Penguin: `10`
    Dog: `20`
    Cat: `30`
    Fish: `40`
    Dragon: `50`
    Hyena: `60`
    Femboy: `10000`";

    match convec[1].as_str() {
        "enrol" => add_self(client, db).await,
        "exit" => remove_self(client, db).await,
        "check" => query_self(client, db).await,
        "dev" => dev_patterns(client, db).await,
        "buy" => buy_pet(client, db).await,
        "shop" => client.message().sender(shop).await,
        _ => {
            client.message().sender("**Invalid command**").await;
        }
    };
}

async fn buy_pet(client: &Do, db: Collection<TProfile>) {
    if !user_exist(&db, &client.input_message.author).await {
        client.message().sender("**User does not exist!**").await;
        return;
    };

    let user = db
        .find_one(doc!("user_id": &client.input_message.author), None)
        .await
        .unwrap()
        .unwrap();

    let convec = client.input().convec();

    // ?t buy frog

    let animal = match Animal::to_enum(&convec[2]) {
        None => {
            client.message().sender("**invalid animal**").await;
            return;
        }
        Some(a) => a,
    };

    if user.money < Animal::cost(&animal) {
        client
            .message()
            .sender("**Cannot buy, not enough coins**")
            .await;
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

async fn remove_self(client: &Do, db: Collection<TProfile>) {
    if !user_exist(&db, &client.input_message.author).await {
        client.message().sender("**User does not exist!**").await;
        return;
    };

    db.delete_one(doc!("user_id": &client.input_message.author), None)
        .await
        .unwrap();

    client.message().sender("**user removed**").await;
}

async fn dev_patterns(client: &Do, db: Collection<TProfile>) {
    if !client.auth.sudoers.contains(&client.input_message.author) {
        client
            .message()
            .sender("**You are not authorised to use dev commands**")
            .await;
        return;
    };
    // ?t dev newday

    match client.input().convec()[2].as_str() {
        "newday" => newday(client, db).await,
        _ => {
            client.message().sender("**Invalid command**").await;
        }
    }
}

async fn newday(client: &Do, db: Collection<TProfile>) {
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
    client.message().sender(":cat_sussy:  :thumbsup: ").await;
}

fn coin_calc(profile: &TProfile) -> u32 {
    let mut fin = 0;

    for x in profile.animals.iter() {
        fin += Animal::value(x);
    }
    fin
}
async fn query_self(client: &Do, db: Collection<TProfile>) {
    if !user_exist(&db, &client.input().author()).await {
        client.message().sender("**User doesn't exist!**").await;
        return;
    };

    let res = match db
        .find_one(doc!("user_id": client.input().author()), None)
        .await
    {
        Ok(a) => mongo_formatting(a),
        Err(_) => String::from("**Could not find user!**"),
    };

    client.message().sender(&res).await;
}

fn mongo_formatting(input: Option<TProfile>) -> String {
    let input = match input {
        Some(a) => a,
        None => return String::from("None"),
    };

    let mut json = Printable::new();

    for x in input.animals {
        json.new_animal(x);
    }

    json.coins = input.money;

    let a = serde_json::to_string_pretty(&json).unwrap();

    format!("```json\n{a}\n```")
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Printable {
    animals: PrintableAnimals,
    coins: u32,
}

impl Printable {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_animal(&mut self, animal: Animal) -> Self {
        match animal {
            Animal::Void => self.animals.void += 1,
            Animal::Penguin => self.animals.penguin += 1,
            Animal::Dog => self.animals.dog += 1,
            Animal::Cat => self.animals.cat += 1,
            Animal::Fish => self.animals.fish += 1,
            Animal::Dragon => self.animals.dragon += 1,
            Animal::Hyena => self.animals.hyena += 1,
            Animal::Femboy => self.animals.femboy += 1,
        }
        self.to_owned()
    }
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PrintableAnimals {
    #[serde(skip_serializing_if = "zero")]
    void: u32,
    #[serde(skip_serializing_if = "zero")]
    penguin: u32,
    #[serde(skip_serializing_if = "zero")]
    dog: u32,
    #[serde(skip_serializing_if = "zero")]
    cat: u32,
    #[serde(skip_serializing_if = "zero")]
    fish: u32,
    #[serde(skip_serializing_if = "zero")]
    dragon: u32,
    #[serde(skip_serializing_if = "zero")]
    hyena: u32,
    #[serde(skip_serializing_if = "zero")]
    femboy: u32,
}

fn zero(t: &u32) -> bool {
    t == &0
}
async fn add_self(client: &Do, db: Collection<TProfile>) {
    // ?t add_self
    if user_exist(&db, &client.input().author()).await {
        client.message().sender("**User already exists**").await;
        return;
    };
    let profile = TProfile::new(&client.input().author());

    let res = match db.insert_one(profile, None).await {
        Err(e) => format!("**Failed inserting user!**\n{e}"),

        Ok(_) => format!("**Successfully added user <@{}>**", client.input().author()),
    };

    client.message().sender(&res).await;
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
    Femboy,
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
            Animal::Femboy => 2000,
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
            "femboy" | "Femboy" => Some(Animal::Femboy),

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
            Animal::Femboy => 10000,
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
            Animal::Femboy => "Femboy",
        }
        .to_string()
    }
}
