use serde::{Deserialize, Serialize};

use crate::types::api::error::APIError;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl PaginationQuery {
    pub fn get_page(&self) -> Result<u64, APIError> {
        match self.page {
            Some(page) => {
                if page < 1 {
                    Err(APIError::BadRequest(
                        "Invalid page: cannot be less than 1.".to_string(),
                    ))
                } else {
                    Ok(page)
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
        match self.page_size {
            Some(page_size) => {
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
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    pub pagination: PaginationData,
    pub data: Vec<T>,
}
