// api/models.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 分页参数
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationParams {
    #[schema(example = 20, default = 20, minimum = 1, maximum = 100)]
    pub limit: Option<i32>,
    #[schema(example = 0, default = 0, minimum = 0)]
    pub offset: Option<i32>,
}

impl PaginationParams {
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(20).min(100).max(1)
    }

    pub fn offset(&self) -> i32 {
        self.offset.unwrap_or(0).max(0)
    }
}

/// 分页响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationResponse {
    pub limit: i32,
    pub offset: i32,
    pub count: usize,
}

/// 错误响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// 香火类型信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IncenseTypeInfo {
    #[schema(example = "basic")]
    pub id: String,
    #[schema(example = "清香")]
    pub name: String,
    #[schema(example = "Basic Incense")]
    pub name_en: String,
    #[schema(example = 0.01)]
    pub price: f64,
    #[schema(example = 1)]
    pub merit_points: i32,
    #[schema(example = "Simple and pure, for daily devotion")]
    pub description: String,
    pub image: String,
    #[schema(example = 10)]
    pub daily_limit: i32,
}

/// 捐赠等级信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DonationTierInfo {
    #[schema(example = "bronze")]
    pub tier: String,
    #[schema(example = "铜牌信士")]
    pub name: String,
    #[schema(example = "Bronze Devotee")]
    pub name_en: String,
    #[schema(example = 0.05)]
    pub min_amount: f64,
    #[schema(example = 65)]
    pub merit_points: i32,
    #[schema(example = "入门功德铜章 NFT")]
    pub badge: String,
    pub benefits: Vec<String>,
}

/// 捐赠交易请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DonationTransactionRequest {
    #[schema(example = "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw")]
    pub user_pubkey: String,
    #[schema(example = 0.1)]
    pub amount_sol: f64,
    #[schema(example = "bronze")]
    pub tier: String,
    #[schema(example = "3m4YtRxKKdYj...")]
    pub transaction_signature: String,
}

/// 成功响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}
