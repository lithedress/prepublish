use aide::{
    axum::{routing, ApiRouter},
    openapi::{Header, ParameterSchemaOrContent, ReferenceOr, SchemaObject},
};
use axum::{debug_handler, extract::State, http::StatusCode};
use axum_jsonschema::Json;
use notice::email::{Address, AddressDef};
use schemars::{schema::Schema, schema_for, JsonSchema};
use serde::Deserialize;

use crate::{mongo_entities::profile::Profile, state::AppState};

use super::{
    super::common::{
        auth::AuthInfoStorage,
        docs,
        err::{Error, Result},
    },
    profile::Res,
    tools,
};

#[derive(JsonSchema)]
#[derive(Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
struct LoginBody {
    #[schemars(with = "AddressDef")]
    email: Address,
    #[schemars(title = "Password", length(min = 1, max = 72))]
    password: String,
}

#[debug_handler]
async fn login(
    mut auth_info_storage: AuthInfoStorage,
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<(StatusCode, Res)> {
    let email = body.email;
    let account = tools::try_find_account(&state.sql_db, &email.to_string())
        .await?
        .ok_or(Error::NotFound(format!("no account with email {}", email)))?;
    let salt = account.salt.into_bytes();
    if tools::get_hash(state.hash_cost, salt, body.password)
        .await?
        .to_vec()
        != account.password_hash
    {
        return Err(Error::BadReqest("wrong password".to_string()));
    }
    let model_profile = Profile::get(state.mongo_db, &email)
        .await?
        .ok_or(Error::NotFound(format!("no profile with email {}", email)))?;
    auth_info_storage.store(
        model_profile._id,
        account.is_administrator,
        account.is_editor,
    )?;
    Ok((StatusCode::CREATED, Json(model_profile.into())))
}

#[debug_handler]
async fn logout(mut auth_info_storage: AuthInfoStorage) {
    auth_info_storage.destroy()
}

fn tag(op: aide::transform::TransformPathItem) -> aide::transform::TransformPathItem {
    op.tag("log in and out")
}

pub(super) fn route() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route_with(
            "/login",
            routing::post_with(login, |op| {
                op.summary("log in")
                    .default_response_with::<(), _>(|mut res| {
                        res.inner().headers.insert(
                            "Set-Cookie".to_string(),
                            ReferenceOr::Item(Header {
                                description: None,
                                style: Default::default(),
                                required: false,
                                deprecated: None,
                                format: ParameterSchemaOrContent::Schema(SchemaObject {
                                    json_schema: Schema::Object(schema_for!(String).schema),
                                    external_docs: None,
                                    example: Some(
                                        "sid=encrypted; Expires=Thu, 13 Apr 2023 10:00:02 GMT"
                                            .into(),
                                    ),
                                }),
                                example: None,
                                examples: Default::default(),
                                extensions: Default::default(),
                            }),
                        );
                        res.description("return a cookie")
                    })
            }),
            tag,
        )
        .api_route_with(
            "/logout",
            routing::delete_with(logout, |op| {
                op.summary("log out")
                    .security_requirement(docs::SECURITY_SCHEME_NAME)
            }),
            tag,
        )
}
