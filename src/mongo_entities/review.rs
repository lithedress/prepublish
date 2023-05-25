use async_trait::async_trait;
use crud::Countable;
use crud_derive::{Countable, Viewable};
use mongo::{
    attached::{Attached, AttachedContent},
    entity::{field, Entity, Index, Indexes},
    oid::{ObjectId, ObjectIdDef},
    MongoDatabase, MongoResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Countable)]
#[derive(Viewable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Review {
    #[viewable(serialize_with = "oid::serialize_object_id_as_hex_string")]
    #[schemars(title = "Version ID", with = "ObjectIdDef")]
    pub(crate) version_id: ObjectId,
    #[viewable]
    #[schemars(
        title = "Criticism",
        description = "This reviewer think that this version still has this drawbacks."
    )]
    pub(crate) criticism: String,
    #[viewable]
    #[schemars(
        title = "Judgement",
        description = "Do this reviewer think that this version should be passed or not?"
    )]
    pub(crate) judgement: bool,
}

#[async_trait]
impl AttachedContent for Review {
    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn schema_name() -> &'static str {
        Self::singular()
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(field!((data in Entity<Attached<Review>>).(content in Attached<Review>).(version_id in Review))).with_key(field!(created_at in Entity<Attached<Review>>)))
    }

    async fn windup(_db: MongoDatabase, _id: &Entity<Attached<Self>>) -> MongoResult<()> {
        Ok(())
    }
}
