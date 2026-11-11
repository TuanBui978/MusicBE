use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Dùng chung cho mọi API tìm kiếm (Songs, Artists, Albums...)
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize)]
pub struct ApiListResponse<T> {
    pub success: bool,
    pub page: i64,
    pub page_size: i64,
    pub data: Vec<T>,
}

pub struct Pager {
    pub page: i64,
    pub limit: i64,
    pub offset: i64,
}

impl PaginationParams {
    pub fn to_pager(&self) -> Pager {
        let page = self.page.unwrap_or(1).max(1);
        let limit = self.page_size.unwrap_or(20).clamp(1, 50);
        let offset = (page - 1) * limit;

        Pager {
            page,
            limit,
            offset,
        }
    }
}

impl Pager {
    pub fn into_response<T>(self, data: Vec<T>) -> ApiListResponse<T> {
        ApiListResponse {
            success: true,
            page: self.page,
            page_size: self.limit,
            data,
        }
    }

    pub fn is_valid_page(&self) -> AppResult<()> {
        if self.page <= 0 {
            return Err(AppError::BadRequest("Page < 1".to_string()));
        }
        Ok(())
    }
}
