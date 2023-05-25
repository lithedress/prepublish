use aide::axum::{routing, ApiRouter};
use async_trait::async_trait;
use axum::body::Bytes;
use axum::{
    body::StreamBody,
    debug_handler,
    extract::{Path, State},
    http::HeaderName,
};
use axum_jsonschema::Json;
use crud::{Countable, Viewable};
use futures_util::Stream;
use mongo::entity::update::Update;
use mongo::oid::ObjectId;
use mongo::owned::Owned;
use mongo::{
    attached::Attached,
    entity::{Entity, EntityView},
    oid::ObjectIdDef,
};
use serde::Serialize;
use serde_json::Value;
use schemars::JsonSchema;

use crate::mongo_entities::profile::Profile;
use crate::mongo_entities::review::Review;
use crate::mongo_entities::thesis::Thesis;
use crate::mongo_entities::version::ReviewPattern;
use super::common::{docs, notice};
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

pub(super) struct ShowAuth;

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
                } else if let VersionState::Reviewing(Reviewing { remainder_ids, .. }) =
                    &model.data.content.state
                {
                    Ok(remainder_ids.contains(&auth_info.id))
                } else if let Some(thesis) = model.data.content.thesis(db.clone()).await? {
                    Ok(thesis.data.content.intro.author_ids.contains(&auth_info.id))
                } else {
                    Ok(false)
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

#[debug_handler]
async fn source(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path((id, index)): Path<(ObjectIdDef, usize)>,
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
    file::download_file(
        state.mongo_db,
        version
            .data
            .content
            .source_ids
            .get(index)
            .ok_or(Error::BadReqest("no such source file".to_string()))?
            .to_owned(),
    )
    .await
    .map_err(Error::from)
}

type Res = Json<EntityView<<Attached<Version> as Viewable>::View>>;

#[debug_handler]
async fn edit(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
    Json(reviewing): Json<Reviewing>,
) -> Result<Res> {
    let id = id.unpack();
    if !auth_info.permitted(Permission::Publishing) {
        return Err(Error::Forbidden("you are not a editor".to_string()));
    }
    let version = <Entity<Attached<Version>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound("cannot get version entity".to_string()))?;
    if let VersionState::Uploaded = version.data.content.state {
        for &reviewer_id in &reviewing.remainder_ids {
            let reviewer = <Entity<Profile>>::try_find_one_by_id(state.mongo_db.clone(), reviewer_id)
                .await
                .map_err(Error::from)?
                .ok_or(Error::BadReqest(format!(
                    "invalid reviewer id {}",
                    reviewer_id
                )))?;
            tokio::spawn(notice::send_email(state.clone(), reviewer, "new review task", ""));
        }
        <Entity<Attached<Version>>>::try_find_one_and_update_by_id(
            state.mongo_db,
            id,
            Version::set_state(Update::default(), VersionState::Reviewing(reviewing))
                .map_err(Error::from)?,
        )
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound("cannot get updated version".to_string()))
        .map(|v| Json(v.into()))
    } else {
        Err(Error::BadReqest("edited version".to_string()))
    }
}

#[debug_handler]
async fn adjudge(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
    Json(judgement): Json<bool>,
) -> Result<Res> {
    let id = id.unpack();
    if !auth_info.permitted(Permission::Publishing) {
        return Err(Error::Forbidden("you are not a editor".to_string()));
    }
    let version = <Entity<Attached<Version>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound("cannot get version entity".to_string()))?;
    match version.data.content.state {
        VersionState::Reviewing(Reviewing {
            pattern: ReviewPattern::Editor,
            remainder_ids,
        }) if !remainder_ids.is_empty() => Err(Error::BadReqest("reviewing".to_string())),
        VersionState::Reviewing(Reviewing {
            pattern: ReviewPattern::Editor,
            ..
        })
        | VersionState::Uploaded => {
            <Entity<Attached<Version>>>::try_find_one_and_update_by_id(state.mongo_db.clone(), id, {
                if judgement {
                    <Entity<Owned<Thesis>>>::set_visibility(
                        state.mongo_db,
                        version.data.content.thesis_id,
                        true,
                    )
                    .await
                    .map_err(Error::from)?;
                }
                Version::set_state(Update::default(), VersionState::Passed(judgement))?
            }).await?.ok_or(Error::NotFound("cannot get updated version".to_string())).map(|v|Json(v.into()))
        }
        _ => Err(Error::BadReqest("cannot adjudge this version".to_string())),
    }
}

#[derive(JsonSchema)]
#[derive(Serialize)]
struct ReviewRes {
    #[schemars(with = "ObjectIdDef")]
    id: ObjectId,
    count: usize,
}

#[debug_handler]
async fn review(
    auth_info: AuthInfo,
    State(state): State<AppState>,
    Path(id): Path<ObjectIdDef>,
    Json(review): Json<Review>,
) -> Result<Json<ReviewRes>> {
    let id = id.unpack();
    let version = <Entity<Attached<Version>>>::try_find_one_by_id(state.mongo_db.clone(), id)
        .await
        .map_err(Error::from)?
        .ok_or(Error::NotFound("cannot get version entity".to_string()))?;
    match version.data.content.state {
        VersionState::Reviewing(Reviewing {
            remainder_ids,
            pattern,
        }) if remainder_ids.contains(&auth_info.id) => {
            let judgement = review.judgement.clone();
            let review_id = <Entity<Attached<Review>>>::insert_one(
                state.mongo_db.clone(),
                Attached {
                    creator_id: Some(auth_info.id),
                    content: review,
                },
            )
            .await
            .map_err(Error::from)?
            .ok_or(Error::NotFound("cannot get reviewer id".to_string()))?;
            let remainder_count = remainder_ids.len() - 1;
            if let (0, true, ReviewPattern::Reviewer) =
                (remainder_count.clone(), judgement, pattern)
            {
                let mut judgement = true;
                for review_id in version.data.content.review_ids {
                    let review = <Entity<Attached<Review>>>::try_find_one_by_id(
                        state.mongo_db.clone(),
                        review_id,
                    )
                    .await
                    .map_err(Error::from)?;
                    if let Some(Entity {
                        data: Attached {
                            content: Review {
                                judgement: false, ..
                            },
                            ..
                        },
                        ..
                                }) = review
                    {
                        judgement = false;
                        break;
                    }
                }
                if judgement {
                    <Entity<Owned<Thesis>>>::set_visibility(
                        state.mongo_db.clone(),
                        version.data.content.thesis_id,
                        true,
                    )
                    .await
                    .map_err(Error::from)?;
                }
            }
            Ok(Json(ReviewRes {id: review_id, count: remainder_count}))
        }
        _ => Err(Error::Forbidden(
            "no permission to reviewing it".to_string(),
        )),
    }
}

fn tag(op: aide::transform::TransformPathItem) -> aide::transform::TransformPathItem {
    op.tag(Version::plural())
}

fn add_parameter_id(op: aide::transform::TransformPathItem) -> aide::transform::TransformPathItem {
    docs::add_one_parameter(
        op,
        "id".to_string(),
        Some("版本ID".to_string()),
        Some(Value::String(ObjectId::new().to_hex())),
    )
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        &format!("/{}", Version::plural()),
        ApiRouter::new()
            .api_route_with(
                "/:id",
                routing::get_with(handlers::show_object::<ShowAuth>, |op| {
                    op.summary("show version information")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                }),
                |op| add_parameter_id(tag(op)),
            )
            .api_route_with(
                "/:id/release",
                routing::get_with(release, |op| {
                    op.summary("show version information")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Bytes, _>(docs::require_cookie::<Bytes>)
                }),
                |op| add_parameter_id(tag(op)),
            )
            .api_route_with(
                "/:id/source/:index",
                routing::get_with(release, |op| {
                    op.summary("show version information")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Bytes, _>(docs::require_cookie::<Bytes>)
                }),
                |op| {
                    docs::add_one_parameter(
                        add_parameter_id(tag(op)),
                        "index".to_string(),
                        Some("the n-th source file".to_string()),
                        Some(Value::Number(0.into())),
                    )
                },
            )
            .api_route_with(
                "/:id/edit",
                routing::patch_with(edit, |op| {
                    op.summary("ask other users to review this version")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                }),
                |op| add_parameter_id(tag(op)),
            )
            .api_route_with(
                "/:id/adjudge",
                routing::patch_with(adjudge, |op| {
                    op.summary("pass or reject a version manually")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Res, _>(docs::require_cookie::<Res>)
                }),
                |op| add_parameter_id(tag(op)),
            )
            .api_route_with(
                "/:id/review",
                routing::patch_with(review, |op| {
                    op.summary("review a version")
                        .security_requirement(docs::SECURITY_SCHEME_NAME)
                        .default_response_with::<Json<ReviewRes>, _>(docs::require_cookie::<Json<ReviewRes>>)
                }),
                |op| add_parameter_id(tag(op)),
            )
    )
}
