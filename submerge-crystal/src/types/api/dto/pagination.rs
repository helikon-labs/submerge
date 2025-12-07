use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::types::api::error::APIError;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    pub page: Option<String>,
    pub page_size: Option<String>,
}

impl PaginationQuery {
    pub fn get_page(&self) -> Result<u64, APIError> {
        match &self.page {
            Some(page) => match page.parse() {
                Ok(page) => {
                    if page < 1 {
                        Err(APIError::BadRequest(
                            "Invalid page: cannot be less than 1.".to_string(),
                        ))
                    } else {
                        Ok(page)
                    }
                }
                _ => Err(APIError::BadRequest(
                    "Invalid page: expected integer.".to_string(),
                )),
            },
            None => Ok(1),
        }
    }

    pub fn get_page_size(
        &self,
        default_page_size: u64,
        max_page_size: u64,
    ) -> Result<u64, APIError> {
        match &self.page_size {
            Some(page_size) => match page_size.parse() {
                Ok(page_size) => {
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
                _ => Err(APIError::BadRequest(
                    "Invalid page size: expected integer.".to_string(),
                )),
            },
            None => Ok(default_page_size),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationData {
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    pub pagination: PaginationData,
    pub data: Vec<T>,
}
