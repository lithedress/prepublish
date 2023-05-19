use aide::axum::{routing, ApiRouter};
use async_trait::async_trait;
use axum_jsonschema::Json;
use crud::Viewable;
use mongo::{
    entity::{update::SettableData, Entity},
    oid::ObjectIdDef,
    owned::Owned,
};

use crate::{
    mongo_entities::paper_collection::{
        category::Category, magazine::Magazine, PaperCollection, PaperCollectionDetail,
    },
    state::AppState,
};

use super::common::{
    auth::{AuthInfo, Permission},
    docs,
    err::{Error, Result},
    handlers::{self, DeleteCfg, InsertCfg, SetCfg, ShowCfg},
};

struct InsertAuth<D: PaperCollectionDetail> {
    phantom: std::marker::PhantomData<D>,
}

#[async_trait]
impl<D: PaperCollectionDetail> InsertCfg for InsertAuth<D> {
    type OC = PaperCollection<D>;

    async fn authenticate(
        _auth_info: super::common::auth::AuthInfo,
        db: mongo::MongoDatabase,
        post: &<Self::OC as mongo::owned::OwnedContent>::Post,
    ) -> super::common::err::Result<()> {
        <Entity<Owned<PaperCollection<Category>>>>::include(db, &post.category_ids)
            .await?
            .then_some(())
            .ok_or(Error::BadReqest("invalid catagoriy id".to_string()))
    }
}

fn authenticate<D: PaperCollectionDetail>(
    auth_info: AuthInfo,
    model: &Entity<Owned<PaperCollection<D>>>,
) -> bool {
    auth_info.permitted(Permission::Managing) || model.data.owner_id == auth_info.id
}

struct ShowAuth<D: PaperCollectionDetail> {
    phantom: std::marker::PhantomData<D>,
}

#[async_trait]
impl<D: PaperCollectionDetail> ShowCfg for ShowAuth<D> {
    type DV = <Owned<PaperCollection<D>> as Viewable>::View;

    type D = Owned<PaperCollection<D>>;

    async fn authenticate(
        auth_info: AuthInfo,
        _db: mongo::MongoDatabase,
        model: &Entity<Self::D>,
    ) -> Result<bool> {
        Ok(model.data.is_public || authenticate(auth_info, model))
    }
}

struct PatchAuth<D: PaperCollectionDetail> {
    phantom: std::marker::PhantomData<D>,
}

#[async_trait]
impl<D: PaperCollectionDetail> SetCfg for PatchAuth<D> {
    type OC = PaperCollection<D>;

    async fn authenticate(
        auth_info: AuthInfo,
        db: mongo::MongoDatabase,
        model: &Entity<Owned<Self::OC>>,
        patch: &<Owned<Self::OC> as SettableData>::P,
    ) -> Result<bool> {
        if let Some(category_ids) = &patch.category_ids {
            if !<Entity<Owned<PaperCollection<Category>>>>::include(db, category_ids).await? {
                return Err(Error::BadReqest("invalid catagoriy id".to_string()));
            }
        }
        Ok(authenticate(auth_info, model))
    }
}

struct DeleteAuth<D: PaperCollectionDetail> {
    phantom: std::marker::PhantomData<D>,
}

#[async_trait]
impl<D: PaperCollectionDetail> DeleteCfg for DeleteAuth<D> {
    type Cd = PaperCollection<D>;

    async fn authenticate(
        auth_info: AuthInfo,
        _db: mongo::MongoDatabase,
        model: &Entity<Owned<Self::Cd>>,
    ) -> Result<bool> {
        Ok(authenticate(auth_info, model))
    }
}

type Res<D> = Json<<Entity<Owned<PaperCollection<D>>> as Viewable>::View>;

fn tag<D: PaperCollectionDetail>(
    op: aide::transform::TransformPathItem,
) -> aide::transform::TransformPathItem {
    op.tag(D::plural())
}

fn nest<D: PaperCollectionDetail>() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        &format!("/{}", D::collection_name()),
        ApiRouter::new()
            .api_route_with(
                "/",
                routing::post_with(handlers::insert_body::<InsertAuth<D>>, |op| {
                    op.summary(&format!("add a {}", D::singular()))
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Json<ObjectIdDef>, _>(
                            docs::require_cookie::<Json<ObjectIdDef>>,
                        )
                }),
                tag::<D>,
            )
            .api_route_with(
                "/:id",
                routing::get_with(handlers::show_object::<ShowAuth<D>>, |op| {
                    op.summary(&format!("show a {}", D::singular()))
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res<D>, _>(docs::require_cookie::<Res<D>>)
                })
                .patch_with(handlers::set_object::<PatchAuth<D>>, |op| {
                    op.summary(&format!("patch a {}", D::singular()))
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res<D>, _>(docs::require_cookie::<Res<D>>)
                })
                .delete_with(handlers::delete_object::<DeleteAuth<D>>, |op| {
                    op.summary(&format!("delete a {}", D::singular()))
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res<D>, _>(docs::require_cookie::<Res<D>>)
                }),
                |op| {
                    docs::add_one_oid_parameter(
                        tag::<D>(op),
                        "id".to_string(),
                        Some("someone's object id".to_string()),
                    )
                },
            ),
    )
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new().merge(nest::<Magazine>())
}
