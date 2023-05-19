use aide::{
    openapi::{Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr, SchemaObject},
    transform::{TransformPathItem, TransformResponse},
    OperationOutput,
};
use mongo::oid::ObjectId;
use schemars::schema_for;

pub(crate) fn require_cookie<R: OperationOutput>(
    res: TransformResponse<R::Inner>,
) -> TransformResponse<R::Inner> {
    res.description("cookie needed")
}

pub(crate) fn add_one_parameter(
    mut op: TransformPathItem,
    name: String,
    description: Option<String>,
    example: Option<serde_json::Value>,
) -> TransformPathItem {
    op.inner_mut()
        .parameters
        .push(ReferenceOr::Item(Parameter::Path {
            parameter_data: ParameterData {
                name,
                description,
                required: true,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(SchemaObject {
                    json_schema: schema_for!(String).schema.into(),
                    external_docs: None,
                    example: example,
                }),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            style: Default::default(),
        }));
    op
}

pub(crate) fn add_one_oid_parameter(
    op: TransformPathItem,
    name: String,
    description: Option<String>,
) -> TransformPathItem {
    add_one_parameter(
        op,
        name,
        description,
        Some(serde_json::Value::String(ObjectId::new().to_hex())),
    )
}

pub(crate) const SECURITY_SCHEME_NAME: &str = "cookieAuth";
