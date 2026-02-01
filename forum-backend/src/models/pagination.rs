use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

impl Pagination {
    pub fn new(total: i64, limit: i64, offset: i64) -> Self {
        Self {
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            data,
            pagination: Pagination::new(total, limit, offset),
        }
    }
}
