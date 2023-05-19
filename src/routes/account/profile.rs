use aide::axum::{routing, ApiRouter};
use async_trait::async_trait;
use axum::{debug_handler, extract::State};
use axum_jsonschema::Json;
use crud::{Patchable, Viewable};
use mongo::{
    entity::{Entity, EntityView},
    MongoDatabase,
};

use super::super::common::{
    auth::AuthInfo,
    docs,
    err::{Error, Result},
    handlers::{self, ShowCfg},
};
use crate::{
    mongo_entities::profile::{Profile, PublicProfile},
    state::AppState,
};

pub(super) type Res = Json<EntityView<<Profile as Viewable>::View>>;

#[debug_handler]
async fn show(auth_info: AuthInfo, State(state): State<AppState>) -> Result<Res> {
    Ok(Json(
        Entity::<Profile>::try_find_one_by_id(state.mongo_db, auth_info.id)
            .await?
            .ok_or(Error::NotFound("no profile".to_string()))?
            .into(),
    ))
}

struct ShowAuth;

#[async_trait]
impl ShowCfg for ShowAuth {
    type D = Profile;
    type DV = PublicProfile;

    async fn authenticate(
        _session: AuthInfo,
        _db: MongoDatabase,
        _model: &Entity<Profile>,
    ) -> Result<bool> {
        Ok(true)
    }
}

#[debug_handler]
async fn update(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Json(body): Json<<Profile as Patchable>::Patch>,
) -> Result<Res> {
    Ok(Json(
        Entity::set_by_id(state.mongo_db, auth_info.id, body)
            .await?
            .ok_or(Error::NotFound("no object id after update".to_string()))?
            .into(),
    ))
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route_with(
            "/",
            routing::get_with(show, |op| {
                op.summary("show my profile")
                    .security_requirement(docs::SECURITY_SCHEME_NAME)
                    .default_response_with::<Res, _>(docs::require_cookie::<Res>)
            })
            .patch_with(update, |op| {
                op.summary("edit my profile")
                    .security_requirement(docs::SECURITY_SCHEME_NAME)
                    .default_response_with::<Res, _>(docs::require_cookie::<Res>)
            }),
            |op| op.tag("manage my profile"),
        )
        .api_route_with(
            "/:id",
            routing::get_with(handlers::show_object::<ShowAuth>, |op| {
                op.summary("show someone's profile")
                    .default_response::<Json<EntityView<PublicProfile>>>()
            }),
            |op| {
                docs::add_one_oid_parameter(
                    op,
                    "id".to_string(),
                    Some("someone's object id".to_string()),
                )
                .tag("visit other profile")
            },
        )
}
