use async_trait::async_trait;
use axum::extract::{Path, State};
use axum_jsonschema::Json;
use crud::{View, Viewable};
use mongo::{
    entity::{update::SettableData, Data, Entity, EntityView},
    oid::ObjectIdDef,
    owned::{Owned, OwnedContent},
    MongoDatabase,
};

use crate::state::AppState;

use super::{
    auth::AuthInfo,
    err::{Error, Result},
};

#[async_trait]
pub(crate) trait InsertCfg: Send + Sync {
    type OC: OwnedContent;
    async fn authenticate(
        auth_info: AuthInfo,
        db: MongoDatabase,
        post: &<Self::OC as OwnedContent>::Post,
    ) -> Result<()>;
}

pub(crate) async fn insert_body<I: InsertCfg>(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Json(body): Json<<I::OC as OwnedContent>::Post>,
) -> Result<ObjectIdDef> {
    let user_id = auth_info.id;
    I::authenticate(auth_info, state.mongo_db.clone(), &body).await?;
    Entity::<Owned<I::OC>>::insert_one_owned(state.mongo_db.clone(), user_id, body)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound("cannot get inserted id".to_string()))
        .map(ObjectIdDef::pack)
}

#[async_trait]
pub(crate) trait ShowCfg {
    type D: Data;
    type DV: View<Object = Self::D>;
    async fn authenticate(
        auth_info: AuthInfo,
        db: MongoDatabase,
        model: &Entity<Self::D>,
    ) -> Result<bool>;
}

pub(crate) async fn show_object<S: ShowCfg>(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
) -> Result<Json<EntityView<S::DV>>> {
    let id = id.unpack();
    let model = <Entity<S::D>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound(format!("no object with id {}", id)))?;
    if !S::authenticate(auth_info, state.mongo_db, &model).await? {
        Err(Error::Forbidden("no permission".to_string()))
    } else {
        Ok(Json(model.into()))
    }
}

#[async_trait]
pub(crate) trait SetCfg {
    type OC: OwnedContent;
    async fn authenticate(
        auth_info: AuthInfo,
        db: MongoDatabase,
        model: &Entity<Owned<Self::OC>>,
        patch: &<Owned<Self::OC> as SettableData>::P,
    ) -> Result<bool>;
}

pub(crate) async fn set_object<U: SetCfg>(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
    Json(body): Json<<Owned<U::OC> as SettableData>::P>,
) -> Result<Json<<Entity<Owned<U::OC>> as Viewable>::View>> {
    let id = id.unpack();
    let model = <Entity<Owned<U::OC>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound(format!("no object with id {}", id)))?;
    if !U::authenticate(auth_info, state.mongo_db.clone(), &model, &body).await? {
        Err(Error::Forbidden("no permission".to_string()))
    } else {
        <Entity<Owned<U::OC>>>::set_by_id(state.mongo_db, id, body)
            .await
            .map_err(Error::from)?
            .ok_or(Error::NotFound(format!(
                "no object with id {} while update",
                id
            )))
            .map(<Entity<Owned<U::OC>>>::into)
            .map(Json)
    }
}

#[async_trait]
pub(crate) trait DeleteCfg {
    type Cd: OwnedContent;
    async fn authenticate(
        auth_info: AuthInfo,
        db: MongoDatabase,
        model: &Entity<Owned<Self::Cd>>,
    ) -> Result<bool>;
}

pub(crate) async fn delete_object<D: DeleteCfg>(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
) -> Result<Json<u64>> {
    let id = id.unpack();
    let model = <Entity<Owned<D::Cd>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound(format!("no object with id {}", id)))?;
    if !D::authenticate(auth_info, state.mongo_db.clone(), &model).await? {
        Err(Error::Forbidden("no permission".to_string()))
    } else {
        model
            .delete_owneds(state.mongo_db)
            .await
            .map_err(Error::from)
            .map(Json)
    }
}
