use async_trait::async_trait;
use crud::Countable;
use crud_derive::{Countable, Patchable, Postable, Viewable};
use mongo::{
    entity::{field, Entity},
    oid::ObjectId,
    owned::Owned,
    MongoDatabase, MongoResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{magazine::Magazine, PaperCollection, PaperCollectionDetail};

#[derive(Countable)]
#[derive(Viewable)]
#[derive(Patchable)]
#[derive(Postable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Category {}

#[async_trait]
impl PaperCollectionDetail for Category {
    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn schema_name() -> &'static str {
        Self::singular()
    }

    async fn windup(
        db: mongo::MongoDatabase,
        entity: &mongo::entity::Entity<mongo::owned::Owned<super::PaperCollection<Self>>>,
    ) -> mongo::MongoResult<()> {
        <PaperCollection<Self>>::pull_category_ids(db.clone(), entity._id).await?;
        <PaperCollection<Magazine>>::pull_category_ids(db, entity._id)
            .await
            .map(|_| ())
    }
}

impl<D: PaperCollectionDetail> PaperCollection<D> {
    pub(super) async fn pull_category_ids(
        db: MongoDatabase,
        category_id: ObjectId,
    ) -> MongoResult<(u64, u64)> {
        <Entity<Owned<Self>>>::pull_sets(db, field!((data in Entity<Owned<PaperCollection<Category>>>).(content in Owned<PaperCollection<Category>>).(category_ids in PaperCollection<Category>)), category_id).await
    }
}
