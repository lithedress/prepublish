use crud::Patch;
use mongodm::{
    doc, field,
    mongo::{
        bson::{self, Document},
        error,
        options::{FindOneAndUpdateOptions, ReturnDocument},
        Database,
    },
    operator::*,
    prelude::{Bson, ObjectId},
    ToRepository,
};

use super::{BlankData, Data, Entity};

#[derive(Default)]
#[derive(Debug)]
pub struct Update {
    pub set: Document,

    pub add_to_set: Document,
    pub pull: Document,

    pub push: Document,
    pub pop: Document,

    pub inc: Document,
}

impl Update {
    pub fn into_update_document(self) -> Document {
        doc! {
            Set: self.set,
            AddToSet: self.add_to_set,
            Pull: self.pull,
            Push: self.push,
            Pop: self.pop,
            Inc: self.inc,
            "$updatedDate": { field!(updated_at in Entity<BlankData>): true }
        }
    }
}

impl<D: Data> Entity<D> {
    pub async fn try_find_one_and_update(
        db: Database,
        filter: Document,
        update: Update,
    ) -> error::Result<Option<Self>> {
        db.repository::<Self>()
            .find_one_and_update(
                filter,
                update.into_update_document(),
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await
    }

    pub async fn try_find_one_and_update_by_id(
        db: Database,
        id: ObjectId,
        update: Update,
    ) -> error::Result<Option<Self>> {
        Self::try_find_one_and_update(db, doc! {field!(_id in Entity<BlankData>): id}, update).await
    }

    pub async fn update_many(
        db: Database,
        query: Document,
        update: Update,
    ) -> error::Result<(u64, u64)> {
        db.repository::<Self>()
            .update_many(query, update.into_update_document(), None)
            .await
            .map(|r| (r.matched_count, r.modified_count))
    }

    pub async fn update_many_by_ids(
        db: Database,
        ids: impl Send + Sync + Iterator<Item = &ObjectId>,
        update: Update,
    ) -> error::Result<(u64, u64)> {
        Self::update_many(
            db,
            doc! {field!(_id in Entity<BlankData>): {In: ids.collect::<Bson>()}},
            update,
        )
        .await
    }
}

pub trait SettableData: Data {
    type P: Patch;
    fn settable_path() -> &'static str;
}

impl<D: SettableData> Entity<D> {
    pub async fn set_by_id(db: Database, id: ObjectId, patch: D::P) -> error::Result<Option<Self>> {
        Self::try_find_one_and_update_by_id(
            db,
            id,
            Update {
                set: bson::to_document(&patch)?
                    .into_iter()
                    .map(|(k, v)| (format!("{}.{}", D::settable_path(), k), v))
                    .collect(),
                ..Update::default()
            },
        )
        .await
    }
}

impl<D: Data> Entity<D> {
    pub async fn pull_sets(
        db: Database,
        path: &'static str,
        value: ObjectId,
    ) -> error::Result<(u64, u64)> {
        let query = doc! {path: value};
        Self::update_many(
            db,
            query.clone(),
            Update {
                pull: query,
                ..Update::default()
            },
        )
        .await
    }
}
