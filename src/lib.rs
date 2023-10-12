use crate::plugins::pluralkit::data::{Composite, ProfileAlias};
use mongodb::{Collection, Database};
use plugins::{federolt::MessageAlias, pluralkit::data::Profile};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod plugins;

pub type SharedCollection<T> = Arc<RwLock<Collection<T>>>;

#[derive(Debug, Clone)]
pub struct DB {
    pub plural: DBPlural,
    pub federolt: DBFederolt,
    pub aliases: DBAliases,
    pub profilebind: DBBind,
}

#[derive(Debug, Clone)]
pub struct DBPlural(SharedCollection<Profile>);
#[derive(Debug, Clone)]
pub struct DBFederolt(SharedCollection<MessageAlias>);
#[derive(Debug, Clone)]
pub struct DBAliases(SharedCollection<ProfileAlias>);
#[derive(Debug, Clone)]
pub struct DBBind(SharedCollection<Composite>);

pub fn collection_locked<T>(
    db: &Database,
    collection: &str,
) -> Result<SharedCollection<T>, mongodb::error::Error> {
    Ok(Arc::from(RwLock::from(db.collection(collection))))
}

impl DB {
    pub async fn init() -> Result<Self, mongodb::error::Error> {
        let db = easymongo::mongo::Mongo::new()
            .username("username")
            .password("password")
            .database("test")
            .db_generate()
            .await?;
        Ok(Self {
            plural: DBPlural(collection_locked(&db, "plural")?),
            aliases: DBAliases(collection_locked(&db, "alias")?),
            federolt: DBFederolt(collection_locked(&db, "federolt")?),
            profilebind: DBBind(collection_locked(&db, "bind")?),
        })
    }
}
