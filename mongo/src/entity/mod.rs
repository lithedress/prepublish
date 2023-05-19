pub mod update;

use crud::View;
pub use mongodm::{doc, field, CollectionConfig, Index, IndexOption, Indexes};
use mongodm::{
    mongo::{
        bson::{self, Document},
        error, Cursor, Database,
    },
    prelude::MongoFindOptions,
    Model, ToRepository,
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::oid::{self, ObjectId, ObjectIdDef};

pub trait Data:
    'static + Clone + Sized + Send + Sync + Unpin + Serialize + DeserializeOwned + CollectionConfig
{
    fn schema_name() -> &'static str;
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Entity<D: Data> {
    pub _id: ObjectId,
    #[serde(bound = "D: Data")]
    pub data: D,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

impl<D: Data> Model for Entity<D> {
    type CollConf = D;
}

impl<D: Data> Entity<D> {
    fn new(data: D) -> Self {
        Self {
            _id: ObjectId::new(),
            data,
            created_at: bson::DateTime::now(),
            updated_at: bson::DateTime::now(),
        }
    }

    pub async fn insert_one(db: Database, data: D) -> error::Result<Option<ObjectId>> {
        db.repository::<Self>()
            .insert_one(Self::new(data), None)
            .await
            .map(|r| r.inserted_id.as_object_id())
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub(crate) struct BlankData;

impl CollectionConfig for BlankData {
    fn collection_name() -> &'static str {
        unreachable!()
    }
}

impl Data for BlankData {
    fn schema_name() -> &'static str {
        unreachable!()
    }
}

#[derive(JsonSchema)]
#[schemars(bound = "DV: View")]
#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
#[derive(Clone)]
#[derive(Debug)]
pub struct EntityView<DV: View>
where
    DV::Object: Data,
{
    #[schemars(with = "ObjectIdDef")]
    #[serde(serialize_with = "oid::serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    #[serde(bound = "DV: View")]
    pub data: DV,
    pub create_time: chrono::DateTime<chrono::Utc>,
    pub update_time: chrono::DateTime<chrono::Utc>,
}

impl<DV: View> From<Entity<DV::Object>> for EntityView<DV>
where
    DV::Object: Data,
{
    fn from(value: Entity<DV::Object>) -> Self {
        Self {
            id: value._id,
            data: value.data.into(),
            create_time: Default::default(),
            update_time: Default::default(),
        }
    }
}

impl<DV: View> View for EntityView<DV>
where
    DV::Object: Data,
{
    type Object = Entity<DV::Object>;
}

impl<D: Data> Entity<D> {
    pub async fn try_find_one(db: Database, filter: Document) -> error::Result<Option<Self>> {
        db.repository::<Self>().find_one(filter, None).await
    }

    pub async fn find(db: Database, filter: Document) -> error::Result<Cursor<Self>> {
        db.repository::<Self>().find(filter, None).await
    }

    pub async fn find_peak(
        db: Database,
        filter: Document,
        sort: Document,
    ) -> error::Result<Cursor<Self>> {
        db.repository::<Self>()
            .find(filter, Some(MongoFindOptions::builder().sort(sort).build()))
            .await
    }

    pub async fn try_find_one_by_id(db: Database, id: ObjectId) -> error::Result<Option<Self>> {
        Self::try_find_one(db, doc! {field!(_id in Entity<BlankData>): id}).await
    }

    pub async fn include(
        db: Database,
        ids: impl IntoIterator<Item = &ObjectId>,
    ) -> error::Result<bool> {
        for &id in ids {
            if let None = Self::try_find_one_by_id(db.clone(), id).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl<D: Data> Entity<D> {
    pub async fn delete(db: Database, query: Document) -> error::Result<u64> {
        db.repository::<Self>()
            .delete_many(query, None)
            .await
            .map(|result| result.deleted_count)
    }

    pub async fn delete_by_id(db: Database, id: ObjectId) -> error::Result<u64> {
        Self::delete(db, doc! {field!(_id in Entity<BlankData>): id}).await
    }
}
