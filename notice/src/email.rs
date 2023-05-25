 pub use lettre::Address;
use schemars::{
    _serde_json::Value,
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject},
    JsonSchema,
};

const DOMAIN: &str = "HoYoverse.com";

pub struct AddressDef {}

impl JsonSchema for AddressDef {
    fn schema_name() -> String {
        "Address".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                title: Some("Address".to_string()),
                description: Some("A email address.".to_string()),
                examples: vec![
                    Value::String(Address::new("Buer", DOMAIN).unwrap().to_string()),
                    Value::String(Address::new("Akademiya", DOMAIN).unwrap().to_string()),
                ],
                ..Metadata::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("email".to_string()),
            ..SchemaObject::default()
        }
        .into()
    }
}
