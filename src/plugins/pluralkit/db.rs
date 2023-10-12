use super::data::{Multi, Profile};
use crate::plugins::pluralkit::data::ProfileAlias;
use crate::{DBAliases, DBPlural, DB};
use bson::{doc, Document};
use futures_util::StreamExt;
use mongodb::results::UpdateResult;

impl DB {
    pub async fn alias_check_smart(
        &self,
        author: impl Into<String> + Clone,
        alias: impl Into<String>,
    ) -> Result<Option<Profile>, mongodb::error::Error> {
        if let Some(ProfileAlias { id, .. }) =
            self.aliases.alias_check(author.clone(), alias).await?
        {
            self.plural.profile_find(author, id.profile_id).await
        } else {
            Ok(None)
        }
    }
}
impl DBPlural {
    pub async fn profile_find_many(
        &self,
        user_id: impl Into<String>,
        profile_name: Option<impl Into<String>>,
    ) -> Result<Vec<Profile>, mongodb::error::Error> {
        Ok(self
            .0
            .read()
            .await
            .find(
                match profile_name {
                    Some(profile_name) => doc! {
                        "_id.user": user_id.into(),
                        "data.name": profile_name.into(),

                    },
                    None => {
                        doc! {
                            "_id.user": user_id.into(),
                        }
                    }
                },
                None,
            )
            .await?
            .filter_map(|a| async move { a.ok() })
            .collect()
            .await)
    }

    pub async fn profile_find_many_smart<T: ToString>(
        &self,
        user_id: impl Into<String>,
        search_field_generic: T,
    ) -> Result<Multi<Profile>, mongodb::error::Error> {
        let search = search_field_generic.to_string();

        Ok(if let Ok(profile_id) = search.parse::<u32>() {
            Multi::Single(self.profile_find(user_id, profile_id).await?)
        } else {
            let data = self.profile_find_many(user_id, Some(search)).await?;
            match data.len() {
                0 | 1 => Multi::Single(data.first().cloned()),
                _ => Multi::Many(data),
            }
        })
    }

    pub async fn profile_find(
        &self,
        user_id: impl Into<String>,
        profile_id: u32,
    ) -> Result<Option<Profile>, mongodb::error::Error> {
        self.0
            .read()
            .await
            .find_one(
                doc! {
                    "_id.user": user_id.into(),
                    "_id.profile": profile_id,

                },
                None,
            )
            .await
    }

    pub async fn profile_edit(
        &self,
        user_id: impl Into<String>,
        profile_id: u32,
        edit: Document,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        // asumes that profile exists
        self.0
            .write()
            .await
            .update_one(
                doc! {
                    "_id": {
                      "user": user_id.into(),
                        "profile": profile_id,
                    }

                },
                edit,
                None,
            )
            .await
    }

    pub async fn profile_delete(
        &self,
        user_id: impl Into<String>,
        profile_id: u32,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        self.0
            .write()
            .await
            .delete_one(
                doc! {
                    "_id": {
                        "user": user_id.into(),
                        "profile": profile_id,
                    }
                },
                None,
            )
            .await
    }
}

impl DBAliases {
    pub async fn alias_check(
        &self,
        author: impl Into<String>,
        alias: impl Into<String>,
    ) -> Result<Option<ProfileAlias>, mongodb::error::Error> {
        Ok(self
            .0
            .read()
            .await
            .find_one(
                doc! {
                "_id.user": author.into(),
                "alias": alias.into(),
                        },
                None,
            )
            .await?)
    }

    pub async fn alias_create(
        &self,
        alias: impl Into<ProfileAlias>,
    ) -> Result<(), mongodb::error::Error> {
        self.0
            .write()
            .await
            .insert_one(alias.into(), None)
            .await
            .map(|_| ())
    }
    pub async fn alias_delete(
        &self,
        author: impl Into<String>,
        alias: impl Into<String>,
    ) -> Result<(), mongodb::error::Error> {
        self.0
            .write()
            .await
            .delete_one(
                doc! {
                "_id.user": author.into(),
                "alias": alias.into(),
                        },
                None,
            )
            .await
            .map(|_| ())
    }
}

pub fn profile_format(input: Vec<Profile>) -> String {
    let mut buffer = String::from("```json\n");
    for item in input {
        buffer += &format!("\n{}", item.format());
    }
    buffer += "\n```";
    buffer
}
