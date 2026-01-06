pub(crate) fn extrinsic_extra_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "checkNonce": "8362",
            "checkWeight": {},
            "checkGenesis": {},
            "checkMortality": {
                "type": "Mortal84",
                "value": "0"
            },
            "checkTxVersion": {},
            "checkSpecVersion": {},
            "checkMetadataHash": {
                "mode": {
                    "type": "Disabled",
                    "value": []
                }
            },
            "checkNonZeroSender": {},
            "chargeAssetTxPayment": {
                "tip": "0",
                "assetId": null
            }
          }))])
        .description(Some(
            "Extrinsic extras in JSON format - checkNonce, checkGenesis, chargeTransactionPayment, etc.".to_string(),
        ))
        .build()
}
