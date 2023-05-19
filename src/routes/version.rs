use async_trait::async_trait;
use axum::{
    body::StreamBody,
    debug_handler,
    extract::{Path, State},
    http::HeaderName,
};
use crud::Viewable;
use futures_util::Stream;
use mongo::{attached::Attached, entity::Entity, oid::ObjectIdDef};

use crate::{
    mongo_entities::version::{Reviewing, Version, VersionState},
    state::AppState,
};

use super::common::{
    auth::{AuthInfo, Permission},
    err::{Error, Result},
    file,
    handlers::{self, ShowCfg},
};

struct ShowAuth;

#[async_trait]
impl ShowCfg for ShowAuth {
    type D = Attached<Version>;

    type DV = <Attached<Version> as Viewable>::View;

    async fn authenticate(
        auth_info: super::common::auth::AuthInfo,
        db: mongo::MongoDatabase,
        model: &mongo::entity::Entity<Self::D>,
    ) -> super::common::err::Result<bool> {
        match model.data.content.state {
            VersionState::History | VersionState::Passed(true) => Ok(true),
            _ => {
                if auth_info.permitted(Permission::Publishing)
                    || model.data.creator_id == Some(auth_info.id)
                {
                    Ok(true)
                } else {
                    if let VersionState::Reviewing(Reviewing { remainder_ids, .. }) =
                        &model.data.content.state
                    {
                        Ok(remainder_ids.contains(&auth_info.id))
                    } else {
                        if let Some(thesis) = model.data.content.thesis(db.clone()).await? {
                            Ok(thesis.data.content.intro.author_ids.contains(&auth_info.id))
                        } else {
                            Ok(false)
                        }
                    }
                }
            }
        }
    }
}

#[debug_handler]
async fn release(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
) -> Result<(
    [(HeaderName, String); 3],
    StreamBody<impl Stream<Item = std::io::Result<Vec<u8>>> + Sized>,
)> {
    let id = id.unpack();
    let version = <Entity<Attached<Version>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::BadReqest("version not found".to_string()))?;
    ShowAuth::authenticate(auth_info, state.mongo_db.clone(), &version).await?;
    Version::downloads(state.mongo_db.clone(), &version).await?;
    file::download_file(state.mongo_db, version.data.content.release_id)
        .await
        .map_err(Error::from)
}
