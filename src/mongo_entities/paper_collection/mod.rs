pub(crate) mod category;
pub(crate) mod magazine;

use std::collections::BTreeSet;

use async_trait::async_trait;
use crud::{Countable, Patchable, Postable, Viewable};
use crud_derive::{Patchable, Postable, Viewable};
use mongo::{
    entity::{field, Entity},
    oid::{ObjectId, ObjectIdDef},
    owned::{Owned, OwnedContent},
    MongoDatabase, MongoResult,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::category::Category;

#[async_trait]
pub(crate) trait PaperCollectionDetail:
    'static
    + Clone
    + Default
    + Sized
    + Send
    + Sync
    + Unpin
    + Serialize
    + DeserializeOwned
    + Postable
    + Patchable
    + Viewable
    + Countable
{
    fn collection_name() -> &'static str;
    fn schema_name() -> &'static str;
    async fn windup(
        db: MongoDatabase,
        entity: &Entity<Owned<PaperCollection<Self>>>,
    ) -> MongoResult<()>;
}

#[derive(Postable)]
#[derive(Viewable)]
#[derive(Patchable)]
#[schemars(bound = "D: PaperCollectionDetail")]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct PaperCollection<D: PaperCollectionDetail> {
    #[viewable(serialize_with = "oid::serialize_object_id_collection_as_hex_string")]
    #[patchable]
    #[schemars(
        title = "Category IDs",
        description = "This paper collection belongs to these categories.\nDo not repeat.",
        with = "BTreeSet<ObjectIdDef>"
    )]
    pub(crate) category_ids: BTreeSet<ObjectId>,
    #[viewable]
    #[patchable]
    #[schemars(title = "Name")]
    pub(crate) name: String,
    #[viewable]
    #[patchable]
    #[schemars(title = "Description")]
    pub(crate) description: String,
    #[viewable(into)]
    #[patchable(into)]
    #[postable(into)]
    #[serde(bound = "D: PaperCollectionDetail", flatten)]
    pub(crate) detail: D,
}

#[async_trait]
impl<D: PaperCollectionDetail> OwnedContent for PaperCollection<D> {
    type Post = <Self as Postable>::Post;
    type P = <Self as Patchable>::Patch;

    fn collection_name() -> &'static str {
        <D as Countable>::plural()
    }

    fn schema_name() -> &'static str {
        <D as Countable>::singular()
    }

    fn new(submitted: Self::Post) -> Self {
        submitted.into()
    }

    fn settable_path() -> &'static str {
        field!((data in Entity<Owned<PaperCollection<Category>>>).(content in Owned<PaperCollection<Category>>).(detail in PaperCollection<Category>))
    }

    async fn windup(
        db: MongoDatabase,
        entity: &Entity<mongo::owned::Owned<Self>>,
    ) -> MongoResult<()> {
        D::windup(db, entity).await
    }
}
