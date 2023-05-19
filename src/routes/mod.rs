use std::sync::Arc;

use aide::{
    axum::{routing, ApiRouter},
    openapi::{ApiKeyLocation, Info, OpenApi, SecurityScheme},
};
use axum::{Extension, Router};
use axum_jsonschema::Json;

use crate::state::AppState;

mod account;
mod common;
mod paper_collection;
mod thesis;
mod version;

pub(crate) fn new() -> Router<AppState> {
    aide::gen::in_context(|ctx| {
        let mut openapi3 = schemars::gen::SchemaSettings::openapi3();
        openapi3.inline_subschemas = true;
        ctx.schema = openapi3.into_generator()
    });
    let mut open_api = OpenApi::default();
    ApiRouter::new()
        .merge(account::route())
        .merge(paper_collection::route())
        .merge(thesis::route())
        .route(
            "/api.json",
            routing::get(|Extension(api): Extension<Arc<OpenApi>>| async { Json(api) }),
        )
        .finish_api_with(&mut open_api, |api| {
            api.info(Info {
                title: "PrePublish".to_string(),
                description: Some(
                    "预出版与文档评阅，高并发版本，非python实现（性能为C++等级）。".to_string(),
                ),
                version: "0.0.1".to_string(),
                ..Info::default()
            })
            .security_scheme(
                "cookieAuth",
                SecurityScheme::ApiKey {
                    location: ApiKeyLocation::Cookie,
                    name: "sid".to_string(),
                    description: None,
                    extensions: Default::default(),
                },
            )
        })
        .layer(Extension(Arc::new(open_api)))
}
