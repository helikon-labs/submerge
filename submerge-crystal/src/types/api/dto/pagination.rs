use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::types::api::error::APIError;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    pub page: Option<String>,
    pub page_size: Option<String>,
}

impl PaginationQuery {
    pub fn get_page(&self) -> Result<u32, APIError> {
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
        default_page_size: u32,
        max_page_size: u32,
    ) -> Result<u32, APIError> {
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

/// Pagination data for paged responses.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "page": 1,
    "pageSize": 1,
    "total": 4352561,
}))]
pub struct PaginationData {
    /// Current page number. 1-indexed.
    #[schema(minimum = 1, example = 1)]
    pub page: u32,
    /// Number of items per page.
    #[schema(minimum = 1, example = 1)]
    #[schema(example = 1)]
    pub page_size: u32,
    /// Total number of items across all pages.
    #[schema(minimum = 0, example = 10467367)]
    pub total: u64,
}

/// Paged data response.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    /// Pagination data.
    pub pagination: PaginationData,
    /// Data on the current page.
    pub data: Vec<T>,
}
