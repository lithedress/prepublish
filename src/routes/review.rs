use crate::mongo_entities::review::Review;
use crate::mongo_entities::thesis::Thesis;
use crate::mongo_entities::version::Version;
use crate::routes::common::auth::AuthInfo;
use crate::routes::common::err::Error;
use crate::routes::common::handlers::ShowCfg;
use crate::state::AppState;
use aide::axum::{ApiRouter, routing};
use async_trait::async_trait;
use axum_jsonschema::Json;
use crud::{Countable, Viewable};
use mongo::attached::Attached;
use mongo::entity::{Entity, EntityView};
use mongo::MongoDatabase;
use crate::routes::common::{docs, handlers};

struct ShowAuth;

#[async_trait]
impl ShowCfg for ShowAuth {
    type D = Attached<Review>;
    type DV = <Attached<Review> as Viewable>::View;

    async fn authenticate(
        auth_info: AuthInfo,
        db: MongoDatabase,
        model: &Entity<Self::D>,
    ) -> crate::routes::common::err::Result<bool> {
        if let Some(creator_id) = model.data.creator_id {
            if auth_info.id == creator_id {
                return Ok(true);
            }
        }
        let version = <Entity<Attached<Version>>>::try_find_one_by_id(
            db.clone(),
            model.data.content.version_id,
        )
        .await
        .map_err(Error::from)?;
        if let Some(version) = version {
            super::version::ShowAuth::authenticate(auth_info, db, &version).await
        } else {
            Ok(false)
        }
    }
}

type Res = Json<EntityView<<Attached<Review> as Viewable>::View>>;

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new().nest(&format!("/{}", Thesis::plural()), ApiRouter::new().api_route_with("/:id", routing::get_with(handlers::show_object::<ShowAuth>, |op| {
        op.summary("get content of a review")
            .security_requirement(docs::SECURITY_SCHEME_NAME)
            .default_response_with::<Res, _>(docs::require_cookie::<Res>)
    }),|op|{
        docs::add_one_oid_parameter(op.tag(Thesis::plural()), "id".to_string(), Some("review id".to_string()))
    }))
}
