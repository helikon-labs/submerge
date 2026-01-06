pub(crate) fn block_weight_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "normal": {
                "refTime": "0",
                "proofSize": "0"
            },
            "mandatory": {
                "refTime": "361766342408",
                "proofSize": "592668"
            },
            "operational": {
                "refTime": "0",
                "proofSize": "0"
            },
        }))])
        .description(Some(
            "Block weight in JSON format. Schema depends on runtime metadata.".to_string(),
        ))
        .build()
}
