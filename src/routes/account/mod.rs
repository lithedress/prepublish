use aide::axum::{routing, ApiRouter};
use axum::{debug_handler, extract::State, http::StatusCode};
use axum_jsonschema::Json;
use chrono::Utc;
use crud::Postable;
use mongo::{entity::Entity, oid::ObjectIdDef};
use notice::email::{Address, AddressDef};
use passwords::hasher;
use schemars::JsonSchema;
use sea_orm::{prelude::Uuid, ActiveValue, EntityTrait, ModelTrait};
use serde::Deserialize;

use crate::{
    mongo_entities::profile::{Bio, Notification, Profile},
    sql_entities::{account, prelude::Account},
    state::AppState,
};

use super::common::err::{Error, Result};

mod logio;
mod profile;
mod tools;

#[derive(JsonSchema)]
#[derive(Deserialize)]
struct SignupBody {
    #[schemars(with = "AddressDef")]
    email: Address,
    #[schemars(
        title = "Password",
        description = "Hash by yourself!",
        length(min = 1, max = 72)
    )]
    password: String,
    bio: <Bio as Postable>::Post,
}

#[debug_handler]
async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupBody>,
) -> Result<(StatusCode, ObjectIdDef)> {
    let email = body.email;
    let account = tools::try_find_account(&state.sql_db, &email.to_string()).await?;
    if account.is_some() {
        return Err(Error::Conflict(format!(
            "account with {} already exists",
            email
        )));
    }
    if Profile::get(state.mongo_db.clone(), &email)
        .await?
        .is_some()
    {
        return Err(Error::Conflict(format!(
            "profile with {} already exists",
            email
        )));
    }
    let salt = hasher::gen_salt();
    let account = account::ActiveModel {
        email: ActiveValue::Set(email.clone().to_string()),
        salt: ActiveValue::Set(Uuid::from_bytes(salt)),
        password_hash: ActiveValue::Set(
            tools::get_hash(state.hash_cost, salt, body.password)
                .await?
                .into(),
        ),
        is_administrator: ActiveValue::Set(
            Account::find()
                .all(&state.sql_db)
                .await
                .map_err(Error::from)?
                .is_empty(),
        ),
        created_at: ActiveValue::Set(Utc::now().naive_utc()),
        updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        is_editor: ActiveValue::Set(false),
    };
    Account::insert(account)
        .exec(&state.sql_db)
        .await
        .map_err(Error::from)?;
    let oid = <Entity<Profile>>::insert_one(
        state.mongo_db,
        Profile {
            email,
            notice: Notification::default(),
            bio: body.bio.into(),
        },
    )
    .await
    .map_err(Error::from)?
    .ok_or(Error::NotFound("no inserted id".to_string()))?;
    Ok((StatusCode::CREATED, ObjectIdDef::pack(oid)))
}

#[derive(JsonSchema)]
#[derive(Deserialize)]
struct DropoutBody {
    #[schemars(with = "AddressDef")]
    email: Address,
    #[schemars(
        title = "Password",
        description = "Hash by yourself!",
        length(min = 1, max = 72)
    )]
    password: String,
}

#[debug_handler]
async fn dropout(
    State(state): State<AppState>,
    Json(body): Json<DropoutBody>,
) -> Result<Json<u64>> {
    let email = body.email;
    let account = tools::try_find_account(&state.sql_db, &email.to_string())
        .await?
        .ok_or(Error::NotFound(format!("{} doesn't exist!", email)))?;
    let salt = account.salt.into_bytes();
    if tools::get_hash(state.hash_cost, salt, body.password)
        .await?
        .to_vec()
        != account.password_hash
    {
        return Err(Error::BadReqest("wrong password".to_string()));
    }
    let profile = Profile::get(state.mongo_db.clone(), &email)
        .await?
        .ok_or(Error::NotFound(format!("no profile with email {}", email)))?;
    account.delete(&state.sql_db).await.map_err(Error::from)?;
    Profile::delete(state.mongo_db, profile)
        .await
        .map_err(Error::from)
        .map(Json)
}

fn tag(op: aide::transform::TransformPathItem) -> aide::transform::TransformPathItem {
    op.tag("account manage")
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route_with(
            "/signup",
            routing::post_with(signup, |op| {
                op.summary("account register")
                    .response::<201, ObjectIdDef>()
            }),
            tag,
        )
        .api_route_with(
            "/dropout",
            routing::delete_with(dropout, |op| {
                op.summary("delete my account")
                    .default_response_with::<Json<u64>, _>(|res| {
                        res.description("count of deleted accounts, which should be 1")
                    })
            }),
            tag,
        )
        .nest("/", logio::route())
        .nest("/profile", profile::route())
}
