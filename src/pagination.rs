// src/pagination.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: Meta,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub limit: u32,
    pub offset: u32,
    pub count: usize,
}
