pub fn call_args_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "hash": "0xb778a81c1fd06d98b5ba1b37bb274101f7905ad5eca960f56ededf26248c4011",
            "args": {
                "dest": {
                    "type": "Id",
                    "value": "0xc35b9a45aadc8bb998ba7c4d17bda4d7d8e31f90a754a65709d3a3a71ff8fa7a"
                },
                "value": "117284000000"
            }
        }))])
        .description(Some(
            "Call arguments in JSON format. Schema depends on the call's schema definition in the runtime metadata.".to_string(),
        ))
        .build()
}
