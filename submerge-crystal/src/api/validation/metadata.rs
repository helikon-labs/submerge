use crate::types::api::error::APIError;

pub fn parse_spec_version(spec_version: &str) -> Result<u32, APIError> {
    spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })
}

pub fn parse_pallet_index(pallet_index: &str) -> Result<u32, APIError> {
    pallet_index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid pallet index: must be a positive integer.".to_string())
    })
}
