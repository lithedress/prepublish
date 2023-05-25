use std::collections::BTreeSet;

use async_trait::async_trait;
use crud::{Countable, Patchable, Postable};
use crud_derive::{Countable, Patchable, Postable, Viewable};
use mongo::{
    attached::{Attached, AttachedContent},
    entity::{doc, field, Entity},
    oid::{ObjectId, ObjectIdDef},
    owned::{Owned, OwnedContent},
    MongoDatabase, MongoResult,
};
use serde::{Deserialize, Serialize};

use super::{examples, version::Version};

#[derive(Viewable)]
#[derive(Patchable)]
#[derive(Postable)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct ThesisIntroduction {
    #[viewable(serialize_with = "oid::serialize_object_id_collection_as_hex_string")]
    #[patchable]
    #[schemars(
        title = "Magazine IDs",
        description = "This thesis belongs to these magazines.\nDo not repeat.",
        with = "BTreeSet<ObjectIdDef>"
    )]
    pub(crate) magazine_ids: BTreeSet<ObjectId>,
    #[viewable]
    #[patchable]
    #[schemars(title = "DOI", example = "examples::doi")]
    pub(crate) doi: Option<String>,
    #[viewable]
    #[patchable]
    #[schemars(title = "Title", example = "examples::title")]
    pub(crate) title: String,
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Abstraction",
        description = "Very very long.",
        example = "examples::abstraction"
    )]
    pub(crate) abstraction: String,
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Abstraction",
        description = "（硕士一般选3～6个单词或专业术语，博士一般选3～8个单词或专业术语，且中英文关键词必须对应。）",
        example = "examples::keywords"
    )]
    pub(crate) keywords: Vec<String>,
    #[viewable(serialize_with = "oid::serialize_object_id_collection_as_hex_string")]
    #[patchable]
    #[schemars(
        title = "Author IDs",
        description = "The 1st author, the 2nd author...",
        with = "Vec<ObjectIdDef>"
    )]
    pub(crate) author_ids: Vec<ObjectId>,
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Languages",
        description = "Do not repeat.",
        example = "examples::language"
    )]
    pub(crate) language: BTreeSet<String>,
}

#[derive(Countable)]
#[derive(Viewable)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Thesis {
    #[viewable(into)]
    pub(crate) intro: ThesisIntroduction,
    #[viewable]
    #[schemars(title = "Downloads", description = "Just count the release files.")]
    pub(crate) downloads: i32,
}

#[async_trait]
impl OwnedContent for Thesis {
    type Post = <ThesisIntroduction as Postable>::Post;
    type P = <ThesisIntroduction as Patchable>::Patch;

    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn schema_name() -> &'static str {
        Self::singular()
    }

    fn new(submitted: Self::Post) -> Self {
        Self {
            intro: submitted.into(),
            ..Self::default()
        }
    }

    fn settable_path() -> &'static str {
        field!((data in Entity<Owned<Thesis>>).(content in Owned<Thesis>).(intro in Thesis))
    }

    async fn windup(
        db: MongoDatabase,
        entity: &Entity<mongo::owned::Owned<Self>>,
    ) -> MongoResult<()> {
        let mut found = <Entity<Attached<Version>>>::find(db.clone(), doc! { field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(thesis_id in Version)): entity._id}).await?;
        while found.advance().await? {
            let version = found.deserialize_current()?;
            Version::windup(db.clone(), &version).await?;
        }
        Ok(())
    }
}

impl Thesis {
    pub(super) async fn pull_magazine_ids(
        db: MongoDatabase,
        magazine_ids: ObjectId,
    ) -> MongoResult<(u64, u64)> {
        <Entity<Owned<Self>>>::pull_sets(db, field!((data in Entity<Owned<Thesis>>).(content in Owned<Thesis>).(intro in Thesis).(magazine_ids in ThesisIntroduction)), magazine_ids).await
    }

    pub(crate) async fn commit(
        db: MongoDatabase,
        committer_id: ObjectId,
        thesis_id: ObjectId,
        release_id: ObjectId,
        source_ids: Vec<ObjectId>,
    ) -> MongoResult<Option<ObjectId>> {
        let last_version = <Entity<Attached<Version>>>::find_peak(db.clone(), doc! {field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(thesis_id in Version)): thesis_id}, doc! {field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(major_number in Version)): -1, field!((data in Entity<Attached<Version>>).(content in Attached<Version>).(minor_number in Version)): -1}).await?.deserialize_current()?;
        <Entity<Attached<Version>>>::insert_one(
            db,
            Attached {
                creator_id: Some(committer_id),
                content: Version {
                    thesis_id,
                    release_id,
                    source_ids,
                    major_number: last_version.data.content.major_number,
                    minor_number: last_version.data.content.minor_number + 1,
                    ..Default::default()
                },
            },
        )
        .await
    }
}
