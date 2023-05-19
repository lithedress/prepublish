use std::collections::BTreeSet;

use async_trait::async_trait;
use crud::Countable;
use crud_derive::{Countable, Viewable};
use mongo::{
    attached::{Attached, AttachedContent},
    entity::{doc, field, Entity, Index, IndexOption, Indexes},
    oid::{ObjectId, ObjectIdDef},
    owned::Owned,
    MongoDatabase, MongoResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{review::Review, thesis::Thesis};

#[derive(Viewable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) enum ReviewPattern {
    #[default]
    Editor,
    Reviewer,
}

#[derive(Countable)]
#[derive(Viewable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Reviewing {
    #[viewable(serialize_with = "oid::serialize_object_id_collection_as_hex_string")]
    #[schemars(
        title = "Review IDs",
        description = "Could not repeat.",
        with = "BTreeSet<ObjectIdDef>"
    )]
    pub(crate) remainder_ids: BTreeSet<ObjectId>,
    #[viewable(into)]
    pub(crate) pattern: ReviewPattern,
}

#[derive(Viewable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) enum VersionState {
    #[default]
    Uploaded,
    Reviewing(#[viewable(into)] Reviewing),
    Passed(#[viewable] bool),
    History,
}

#[derive(Countable)]
#[derive(Viewable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Version {
    #[viewable(serialize_with = "oid::serialize_object_id_as_hex_string")]
    #[schemars(title = "Thesis ID", with = "ObjectIdDef")]
    pub(crate) thesis_id: ObjectId,
    #[viewable(serialize_with = "oid::serialize_object_id_as_hex_string")]
    #[schemars(
        title = "Release File ID",
        description = "PDF, or not PDF, that is the question.",
        with = "ObjectIdDef"
    )]
    pub(crate) release_id: ObjectId,
    #[viewable(serialize_with = "oid::serialize_object_id_as_hex_string")]
    #[schemars(
        title = "Source File IDs",
        description = "file_0.c, file_1.rs, file_2.sh...",
        with = "Vec<ObjectIdDef>"
    )]
    pub(crate) source_ids: Vec<ObjectId>,
    #[viewable]
    #[schemars(title = "Major Number.", description = "Congregation to be passed.")]
    pub(crate) major_number: i32,
    #[viewable]
    #[schemars(title = "Minor Number.", description = "A little improvement.")]
    pub(crate) minor_number: i32,
    #[viewable(into)]
    pub(crate) state: VersionState,
    #[viewable(serialize_with = "oid::serialize_object_id_collection_as_hex_string")]
    #[schemars(
        title = "Review IDs",
        description = "Could not repeat.",
        with = "BTreeSet<ObjectIdDef>"
    )]
    pub(crate) review_ids: BTreeSet<ObjectId>,
    #[viewable]
    #[schemars(title = "Downloads", description = "Just count the release.")]
    pub(crate) downloads: i32,
}

#[async_trait]
impl AttachedContent for Version {
    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(thesis_id in Version))).with_key(field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(major_number in Version))).with_key(field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(minor_number in Version))).with_option(IndexOption::Unique))
    }

    fn schema_name() -> &'static str {
        Self::singular()
    }

    async fn windup(db: MongoDatabase, entity: &Entity<Attached<Self>>) -> MongoResult<()> {
        <Entity<Attached<Review>>>::delete(db, doc! {field!((data in Entity<Attached<Review>>).(content in Attached<Review>).(version_id in Review)): entity._id}).await.map(|_|())
    }
}

impl Version {
    pub(crate) async fn thesis(
        &self,
        db: MongoDatabase,
    ) -> MongoResult<Option<Entity<Owned<Thesis>>>> {
        <Entity<Owned<Thesis>>>::try_find_one_by_id(db, self.thesis_id).await
    }

    pub(crate) async fn downloads(
        db: MongoDatabase,
        model: &Entity<Attached<Version>>,
    ) -> MongoResult<Option<Entity<Attached<Version>>>> {
        if let Some(thesis) =
            <Entity<Owned<Thesis>>>::try_find_one_by_id(db.clone(), model.data.content.thesis_id)
                .await?
        {
            <Entity<Owned<Thesis>>>::try_find_one_and_update_by_id(db.clone(), thesis._id, mongo::entity::update::Update { inc: doc! {field!((data in Entity<Owned<Thesis>>).(content in Owned<Thesis>).(downloads in Thesis)): 1}, ..Default::default() }).await?;
            <Entity<Attached<Version>>>::try_find_one_and_update_by_id(db, model._id, mongo::entity::update::Update { inc: doc! {field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(downloads in Version)): 1}, ..Default::default() }).await
        } else {
            Ok(None)
        }
    }
}
