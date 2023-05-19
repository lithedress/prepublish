use aide::openapi::{MediaType, Response};
use aide::OperationOutput;
use axum::response::IntoResponse;
use indexmap::IndexMap;
pub use mongodm::bson::serde_helpers::serialize_object_id_as_hex_string;
pub use mongodm::prelude::ObjectId;
use schemars::_serde_json::Value;
use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Metadata, Schema, SchemaObject};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectIdDef(#[serde(serialize_with = "serialize_object_id_as_hex_string")] ObjectId);

impl ObjectIdDef {
    /// used as URL path parameters
    pub fn unpack(self) -> ObjectId {
        self.0
    }

    // used as HTTP response
    pub fn pack(oid: ObjectId) -> Self {
        Self(oid)
    }
}

impl JsonSchema for ObjectIdDef {
    fn schema_name() -> String {
        "ObjectId".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("Object ID".to_string()),
                description: Some("A 12-byte integer as a hex string.".to_string()),
                examples: vec![
                    Value::String("000000000000000000000000".to_string()),
                    Value::String(ObjectId::new().to_hex()),
                    Value::String("ffffffffffffffffffffffff".to_string()),
                ],
                ..Metadata::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: None,
            ..SchemaObject::default()
        }
        .into()
    }
}

impl IntoResponse for ObjectIdDef {
    fn into_response(self) -> axum::response::Response {
        self.0.to_hex().into_response()
    }
}

impl OperationOutput for ObjectIdDef {
    type Inner = Self;

    fn operation_response(
        _ctx: &mut aide::gen::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        let mut schema = schema_for!(Self).schema;
        Some(Response {
            description: schema.metadata().description.clone().unwrap_or_default(),
            content: IndexMap::from_iter([(
                "application/bson".to_string(),
                MediaType {
                    schema: Some(aide::openapi::SchemaObject {
                        json_schema: schema.into(),
                        example: Some(Value::String(ObjectId::new().to_hex())),
                        external_docs: None,
                    }),
                    ..MediaType::default()
                },
            )]),
            ..Response::default()
        })
    }
}

pub fn serialize_object_id_collection_as_hex_string<'a, S: Serializer>(
    val: impl IntoIterator<Item = &'a ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(val.into_iter().map(|o| o.to_hex()))
}

pub fn serialize_object_id_option_as_hex_string<S: Serializer>(
    val: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match *val {
        Some(ref value) => serializer.serialize_some(&value.to_hex()),
        None => serializer.serialize_none(),
    }
}
