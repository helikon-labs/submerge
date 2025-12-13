pub fn metadata_constant_value_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "refTime": "1600000000000",
            "proofSize": "8388608"
        }))])
        .description(Some(
            "Metadata constant value in JSON format. Schema depends on the definition within the runtime metadata.".to_string(),
        ))
        .build()
}
