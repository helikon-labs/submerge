use serde::{Deserialize, Serialize};

use crate::types::api::error::APIError;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page_number: Option<String>,
    pub page_size: Option<String>,
}

impl PaginationQuery {
    pub fn get_page_number(&self) -> Result<u64, APIError> {
        match self.page_number.as_deref() {
            Some(page_number) => {
                let page_number = page_number.parse::<u64>().map_err(|_| {
                    APIError::BadRequest(
                        "Invalid page_number: must be a positive integer.".to_string(),
                    )
                })?;
                if page_number < 1 {
                    Err(APIError::BadRequest(
                        "Invalid page_number: cannot be less than 1.".to_string(),
                    ))
                } else {
                    Ok(page_number)
                }
            }
            None => Ok(1),
        }
    }

    pub fn get_page_size(
        &self,
        default_page_size: u64,
        max_page_size: u64,
    ) -> Result<u64, APIError> {
        match self.page_size.as_deref() {
            Some(page_size) => {
                let page_size = page_size.parse::<u64>().map_err(|_| {
                    APIError::BadRequest(
                        "Invalid page_size: must be a positive integer.".to_string(),
                    )
                })?;
                if page_size < 1 {
                    Err(APIError::BadRequest(
                        "Invalid page_size: cannot be less than 1.".to_string(),
                    ))
                } else if page_size > max_page_size {
                    Err(APIError::BadRequest(format!(
                        "Invalid page_size: cannot be greater than {max_page_size}."
                    )))
                } else {
                    Ok(page_size)
                }
            }
            None => Ok(default_page_size),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationData {
    pub page_number: u64,
    pub page_size: u64,
    pub total_count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    pub pagination: PaginationData,
    pub data: Vec<T>,
}
