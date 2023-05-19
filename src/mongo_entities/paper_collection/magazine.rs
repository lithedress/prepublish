use std::collections::BTreeSet;

use async_trait::async_trait;
use crud::Countable;
use crud_derive::{Countable, Patchable, Postable, Viewable};
use mongo::{entity::Entity, owned::Owned, MongoDatabase, MongoResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{
    super::{examples, thesis::Thesis},
    PaperCollection, PaperCollectionDetail,
};

#[derive(Postable)]
#[derive(Countable)]
#[derive(Viewable)]
#[derive(Patchable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Magazine {
    #[viewable]
    #[patchable]
    #[schemars(title = "Abbreviations", description = "Do not repeat.")]
    pub(crate) abbr: BTreeSet<String>,
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Languages",
        description = "Do not repeat.",
        example = "examples::language"
    )]
    pub(crate) language: BTreeSet<String>,
    #[viewable]
    #[patchable]
    #[schemars(title = "Minimal Page Number", description = "Less is more.")]
    pub(crate) pages_min: i32,
    #[viewable]
    #[patchable(serde(with = "::serde_with::rust::double_option"))]
    #[schemars(title = "Home Page", example = "examples::homepage")]
    pub(crate) homepage: Option<Url>,
    #[viewable]
    #[patchable(serde(with = "::serde_with::rust::double_option"))]
    #[schemars(title = "Template", example = "examples::template_link")]
    pub(crate) template_link: Option<Url>,
    #[viewable]
    #[patchable(serde(with = "::serde_with::rust::double_option"))]
    #[schemars(title = "Community", example = "examples::community_link")]
    pub(crate) community_link: Option<Url>,
    #[viewable]
    #[patchable]
    #[schemars(title = "Other Information")]
    pub(crate) others: String,
}

#[async_trait]
impl PaperCollectionDetail for Magazine {
    async fn windup(
        db: MongoDatabase,
        entity: &Entity<Owned<PaperCollection<Self>>>,
    ) -> MongoResult<()> {
        Thesis::pull_magazine_ids(db, entity._id).await.map(|_| ())
    }

    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn schema_name() -> &'static str {
        Self::singular()
    }
}
