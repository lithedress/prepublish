use async_trait::async_trait;
use crud::{BlankPatch, BlankSubmitted, Patch, Post, Viewable};
use crud_derive::Viewable;
use mongodm::{
    doc, field,
    mongo::{error, Database},
    CollectionConfig, Index, IndexOption, Indexes,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{
    entity::{update::SettableData, BlankData, Data, Entity, EntityView},
    oid::{ObjectId, ObjectIdDef},
};

#[async_trait]
pub trait OwnedContent:
    'static + Clone + Default + Sized + Send + Sync + Unpin + Serialize + DeserializeOwned + Viewable
{
    type Post: Post;
    type P: Patch;
    fn collection_name() -> &'static str;
    fn schema_name() -> &'static str;
    fn new(submitted: Self::Post) -> Self;
    fn settable_path() -> &'static str;
    async fn windup(db: Database, entity: &Entity<Owned<Self>>) -> error::Result<()>;
}

#[async_trait]
impl OwnedContent for () {
    type Post = BlankSubmitted;

    type P = BlankPatch;

    fn collection_name() -> &'static str {
        unimplemented!()
    }

    fn schema_name() -> &'static str {
        unreachable!()
    }

    fn new(_: Self::Post) -> Self {
        unreachable!()
    }

    fn settable_path() -> &'static str {
        unreachable!()
    }

    async fn windup(_: Database, _: &Entity<Owned<Self>>) -> error::Result<()> {
        unreachable!()
    }
}

#[derive(Viewable)]
#[schemars(bound = "C: OwnedContent", rename = "{C}")]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Owned<C: OwnedContent> {
    #[viewable(serialize_with = "oid::serialize_object_id_as_hex_string")]
    #[schemars(title = "Owner ID", with = "ObjectIdDef")]
    pub owner_id: ObjectId,
    #[viewable]
    pub is_public: bool,
    #[viewable(into)]
    #[serde(bound = "C: OwnedContent")]
    pub content: C,
}

impl<C: OwnedContent> CollectionConfig for Owned<C> {
    fn collection_name() -> &'static str {
        C::collection_name()
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!(owner_id in Owned<()>))
                .with_key(field!(is_public in Owned<()>))
                .with_key(field!(created_at in Entity<BlankData>))
                .with_option(IndexOption::Unique),
        )
    }
}

impl<C: OwnedContent> Data for Owned<C> {
    fn schema_name() -> &'static str {
        C::schema_name()
    }
}

impl<C: OwnedContent> SettableData for Owned<C> {
    fn settable_path() -> &'static str {
        C::settable_path()
    }

    type P = C::P;
}

impl<C: OwnedContent> Viewable for Entity<Owned<C>> {
    type View = EntityView<<Owned<C> as Viewable>::View>;
}

impl<C: OwnedContent> Entity<Owned<C>> {
    pub async fn insert_one_owned(
        db: Database,
        owner_id: ObjectId,
        content: C::Post,
    ) -> error::Result<Option<ObjectId>> {
        Self::insert_one(
            db.clone(),
            Owned {
                owner_id,
                content: C::new(content),
                ..Owned::default()
            },
        )
        .await
    }

    pub async fn set_visibility(
        db: Database,
        id: ObjectId,
        is_public: bool,
    ) -> error::Result<Option<Self>> {
        Self::try_find_one_and_update_by_id(
            db,
            id,
            crate::entity::update::Update {
                set: doc! {field!((data in Entity<Owned<()>>).(is_public in Owned<()>)): is_public},
                ..Default::default()
            },
        )
        .await
    }

    pub async fn delete_owneds(self, db: Database) -> error::Result<u64> {
        C::windup(db.clone(), &self).await?;
        Self::delete_by_id(db, self._id).await
    }

    pub async fn delete_owneds_of_owner(db: Database, owner_id: ObjectId) -> error::Result<u64> {
        let filter = doc! {field!((data in Entity<Owned<()>>).(owner_id in Owned<()>)): owner_id};
        let mut owneds = Self::find(db.clone(), filter.clone()).await?;
        while owneds.advance().await? {
            let owned = owneds.deserialize_current()?;
            C::windup(db.clone(), &owned).await?;
        }
        Self::delete(db, filter).await
    }
}
