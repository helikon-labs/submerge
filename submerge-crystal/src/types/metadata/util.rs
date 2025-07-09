use frame_metadata::decode_different::{DecodeDifferent, DecodeDifferentStr};

pub fn get_decode_different_string(value: &DecodeDifferentStr) -> String {
    match value {
        DecodeDifferent::Encode(name) => name.to_string(),
        DecodeDifferent::Decoded(name) => name.clone(),
    }
}
