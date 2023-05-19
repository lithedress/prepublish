use async_trait::async_trait;
use crud::Viewable;
use crud_derive::Viewable;
use mongodm::{
    doc, field,
    mongo::{error, Database},
    CollectionConfig, Index, Indexes,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{
    entity::{update::Update, BlankData, Data, Entity, EntityView},
    oid::{ObjectId, ObjectIdDef},
};

#[async_trait]
pub trait AttachedContent:
    'static + Clone + Default + Sized + Send + Sync + Unpin + Serialize + DeserializeOwned + Viewable
{
    fn collection_name() -> &'static str;
    fn schema_name() -> &'static str;
    fn indexes() -> Indexes;
    async fn windup(db: Database, entity: &Entity<Attached<Self>>) -> error::Result<()>;
}

#[async_trait]
impl AttachedContent for () {
    fn collection_name() -> &'static str {
        unreachable!()
    }

    fn indexes() -> Indexes {
        unreachable!()
    }

    fn schema_name() -> &'static str {
        unreachable!()
    }

    async fn windup(_: Database, _: &Entity<Attached<Self>>) -> error::Result<()> {
        unreachable!()
    }
}

#[derive(Viewable)]
#[schemars(bound = "C: AttachedContent", rename = "{C}")]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Attached<C: AttachedContent> {
    #[viewable(serialize_with = "oid::serialize_object_id_option_as_hex_string")]
    #[schemars(title = "Creator ID", with = "ObjectIdDef")]
    pub creator_id: Option<ObjectId>,
    #[viewable(into)]
    #[serde(bound = "C: AttachedContent")]
    pub content: C,
}

impl<C: AttachedContent> CollectionConfig for Attached<C> {
    fn collection_name() -> &'static str {
        C::collection_name()
    }

    fn indexes() -> Indexes {
        C::indexes().with(
            Index::new(field!(creator_id in Attached<()>))
                .with_key(field!(created_at in Entity<BlankData>)),
        )
    }
}

impl<C: AttachedContent> Data for Attached<C> {
    fn schema_name() -> &'static str {
        C::schema_name()
    }
}

impl<C: AttachedContent> Viewable for Entity<Attached<C>> {
    type View = EntityView<<Attached<C> as Viewable>::View>;
}

impl<C: AttachedContent> Entity<Attached<C>> {
    pub async fn remove_creator_of_attached(
        db: Database,
        creator_id: ObjectId,
    ) -> error::Result<(u64, u64)> {
        let path = field!((data in Entity<Attached<()>>).(creator_id in Attached<()>));
        Self::update_many(
            db,
            doc! {path: Some(creator_id)},
            Update {
                set: doc! {path: None::<ObjectId>},
                ..Update::default()
            },
        )
        .await
    }
}
