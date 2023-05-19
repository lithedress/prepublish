use std::ops::Not;

use aide::axum::{routing, ApiRouter};
use async_trait::async_trait;
use axum::{
    debug_handler,
    extract::{Multipart, Path, State},
};
use axum_jsonschema::Json;
use crud::{Countable, Viewable};
use mongo::{
    entity::{update::SettableData, Entity, EntityView},
    oid::{ObjectId, ObjectIdDef},
    owned::{Owned, OwnedContent},
};

use crate::{
    mongo_entities::{
        paper_collection::{magazine::Magazine, PaperCollection},
        thesis::Thesis,
    },
    state::AppState,
};

use super::common::{
    auth::{AuthInfo, Permission},
    docs,
    err::{self, Error},
    file,
    handlers::{self, DeleteCfg, InsertCfg, SetCfg, ShowCfg},
};

struct InsertAuth;

#[async_trait]
impl InsertCfg for InsertAuth {
    type OC = Thesis;

    async fn authenticate(
        _auth_info: super::common::auth::AuthInfo,
        db: mongo::MongoDatabase,
        post: &<Self::OC as mongo::owned::OwnedContent>::Post,
    ) -> super::common::err::Result<()> {
        <Entity<Owned<PaperCollection<Magazine>>>>::include(db, &post.magazine_ids)
            .await?
            .then_some(())
            .ok_or(Error::BadReqest("invalid catagoriy id".to_string()))
    }
}

fn authenticate(auth_info: AuthInfo, model: &Entity<Owned<Thesis>>) -> bool {
    auth_info.permitted(Permission::Publishing)
        || model.data.owner_id == auth_info.id
        || model.data.content.intro.author_ids.contains(&auth_info.id)
}

struct ShowAuth;

#[async_trait]
impl ShowCfg for ShowAuth {
    type D = Owned<Thesis>;

    type DV = <Owned<Thesis> as Viewable>::View;

    async fn authenticate(
        auth_info: AuthInfo,
        _db: mongo::MongoDatabase,
        model: &Entity<Self::D>,
    ) -> super::common::err::Result<bool> {
        Ok(model.data.is_public || authenticate(auth_info, model))
    }
}

struct UpdateAuth;

#[async_trait]
impl SetCfg for UpdateAuth {
    type OC = Thesis;

    async fn authenticate(
        auth_info: AuthInfo,
        db: mongo::MongoDatabase,
        model: &Entity<Owned<Self::OC>>,
        patch: &<Owned<Self::OC> as SettableData>::P,
    ) -> super::common::err::Result<bool> {
        if let Some(magazine_ids) = &patch.magazine_ids {
            if !<Entity<Owned<PaperCollection<Magazine>>>>::include(db, magazine_ids).await? {
                return Err(Error::BadReqest("invalid magazine id".to_string()));
            }
        }
        Ok(authenticate(auth_info, model))
    }
}

struct DeleteAuth;

#[async_trait]
impl DeleteCfg for DeleteAuth {
    type Cd = Thesis;

    async fn authenticate(
        auth_info: AuthInfo,
        _db: mongo::MongoDatabase,
        model: &Entity<Owned<Self::Cd>>,
    ) -> super::common::err::Result<bool> {
        Ok(if model.data.is_public {
            auth_info.permitted(Permission::Publishing)
        } else {
            authenticate(auth_info, model)
        })
    }
}

type Res = Json<EntityView<<Owned<Thesis> as Viewable>::View>>;

#[debug_handler]
async fn commit(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
    mut multipart: Multipart,
) -> err::Result<ObjectIdDef> {
    let id = id.unpack();
    let model = <Entity<Owned<Thesis>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await?
        .ok_or(Error::BadReqest("wrong thesis id".to_string()))?;
    if authenticate(auth_info, &model).not() {
        return Err(Error::Forbidden("cannot commit".to_string()));
    }
    let file_ids = file::upload_files(state.mongo_db.clone(), &mut multipart).await?;
    Thesis::commit(
        state.mongo_db,
        auth_info.id,
        id,
        file_ids
            .first()
            .ok_or(Error::BadReqest("at least one file".to_string()))?
            .to_owned(),
        file_ids.into_iter().skip(1).collect(),
    )
    .await
    .map_err(Error::from)?
    .ok_or(Error::NotFound("cannot get new version id".to_string()))
    .map(ObjectIdDef::pack)
}

fn tag(op: aide::transform::TransformPathItem) -> aide::transform::TransformPathItem {
    op.tag(Thesis::plural())
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        &format!("/{}", Thesis::collection_name()),
        ApiRouter::new()
            .api_route_with(
                "/",
                routing::post_with(handlers::insert_body::<InsertAuth>, |op| {
                    op.summary("add a new thesis")
                        .description("not public now")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<ObjectIdDef, _>(
                            docs::require_cookie::<ObjectIdDef>,
                        )
                }),
                tag,
            )
            .api_route_with(
                "/:id",
                routing::get_with(handlers::show_object::<ShowAuth>, |op| {
                    op.summary("get information of a thesis")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                })
                .patch_with(handlers::set_object::<UpdateAuth>, |op| {
                    op.summary("modify information of a thesis")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                })
                .delete_with(handlers::delete_object::<DeleteAuth>, |op| {
                    op.summary("delete a thesis")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                }),
                |op| {
                    docs::add_one_parameter(
                        tag(op),
                        "id".to_string(),
                        Some("thesis id".to_string()),
                        Some(ObjectId::new().to_hex().into()),
                    )
                },
            )
            .api_route_with(
                "/:id/commit",
                routing::post_with(commit, |op| {
                    op.summary("上传新版本")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                }),
                |op| {
                    docs::add_one_parameter(
                        tag(op),
                        "id".to_string(),
                        Some("thesis id".to_string()),
                        Some(ObjectId::new().to_hex().into()),
                    )
                },
            ),
    )
}
