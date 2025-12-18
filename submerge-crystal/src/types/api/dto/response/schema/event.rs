pub fn event_args_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "dispatchInfo": {
                "class": {
                    "type": "Mandatory",
                    "value": []
                },
                "paysFee": {
                    "type": "Yes",
                    "value": []
                },
                "weight": {
                    "proofSize": "0",
                    "refTime": "125000000"
                }
            }
        }))])
        .description(Some(
            "Call arguments in JSON format. Schema depends on the call's schema definition in the runtime metadata.".to_string(),
        ))
        .build()
}
