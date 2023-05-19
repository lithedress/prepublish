use crud::{Countable, Patchable, View, Viewable};
use crud_derive::{Countable, Patchable, Postable, Viewable};
use mongo::{
    attached::Attached,
    bson,
    entity::{
        doc, field, update::SettableData, CollectionConfig, Data, Entity, Index, IndexOption,
        Indexes,
    },
    oid::{ObjectId, ObjectIdDef},
    owned::Owned,
    MongoDatabase, MongoResult,
};
use notice::email::{Address, AddressDef};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    examples,
    paper_collection::{category::Category, magazine::Magazine, PaperCollection},
    review::Review,
    thesis::Thesis,
    version::Version,
};

#[derive(Viewable)]
#[derive(Patchable)]
#[derive(Postable)]
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Bio {
    #[viewable]
    #[patchable]
    #[schemars(title = "Name", example = "examples::profile_name")]
    pub(crate) name: String,
    #[viewable(serialize_with = "oid::serialize_object_id_option_as_hex_string")]
    #[patchable]
    #[schemars(title = "Avatar File ID", with = "ObjectIdDef")]
    pub(crate) avatar_id: Option<ObjectId>,
}

#[derive(Viewable)]
#[derive(Patchable)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Notification {
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Email Notification",
        description = "Get email notifications or not."
    )]
    pub(crate) email: bool,
    #[viewable]
    #[patchable]
    #[schemars(
        title = "Push Notification",
        description = "Get push notifications or not."
    )]
    pub(crate) push: bool,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            email: true,
            push: false,
        }
    }
}

#[derive(Countable)]
#[derive(Viewable)]
#[derive(Patchable)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct Profile {
    #[viewable]
    #[schemars(title = "Email Address", with = "AddressDef")]
    pub(crate) email: Address,
    #[viewable(into)]
    #[patchable(into)]
    #[serde(flatten)]
    pub(crate) notice: Notification,
    #[viewable(into)]
    #[patchable(into)]
    #[serde(flatten)]
    pub(crate) bio: Bio,
}

impl CollectionConfig for Profile {
    fn collection_name() -> &'static str {
        Self::plural()
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!((data in Entity<Profile>).(email in Profile)))
                .with_option(IndexOption::Unique),
        )
    }
}

impl Data for Profile {
    fn schema_name() -> &'static str {
        Self::singular()
    }
}

impl SettableData for Profile {
    type P = <Self as Patchable>::Patch;
    fn settable_path() -> &'static str {
        field!(data in Entity<Profile>)
    }
}

impl Profile {
    pub(crate) async fn get(
        db: MongoDatabase,
        email: &Address,
    ) -> MongoResult<Option<Entity<Self>>> {
        <Entity<Self>>::try_find_one(
            db,
            doc! { field!((data in Entity<Profile>).(email in Profile)): bson::to_bson(email)? },
        )
        .await
    }

    pub(crate) async fn delete(db: MongoDatabase, entity: Entity<Self>) -> MongoResult<u64> {
        <Entity<Owned<Thesis>>>::delete_owneds_of_owner(db.clone(), entity._id).await?;
        <Entity<Owned<PaperCollection<Magazine>>>>::delete_owneds_of_owner(db.clone(), entity._id)
            .await?;
        <Entity<Owned<PaperCollection<Category>>>>::delete_owneds_of_owner(db.clone(), entity._id)
            .await?;
        <Entity<Attached<Review>>>::remove_creator_of_attached(db.clone(), entity._id).await?;
        <Entity<Attached<Version>>>::remove_creator_of_attached(db.clone(), entity._id).await?;
        <Entity<Self>>::delete_by_id(db, entity._id).await
    }
}

#[derive(JsonSchema)]
#[schemars(description = "profile in visitor's view")]
#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct PublicProfile(<Bio as Viewable>::View);

impl From<Profile> for PublicProfile {
    fn from(value: Profile) -> Self {
        Self(value.bio.into())
    }
}

impl View for PublicProfile {
    type Object = Profile;
}
