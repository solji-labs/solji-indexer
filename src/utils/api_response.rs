use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub code: i32,       // 业务码：0 成功；非 0 代表错误
    pub message: String, // 描述
    pub data: Option<T>, // 成功时 Some(..)，失败/未找到时 None
}

#[derive(Debug, Serialize)]
pub struct PageResp<T> {
    pub list: Vec<T>,
    pub total: u64,
    pub page_num: u32,
    pub page_size: u32,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 200,
            message: "ok".into(),
            data: Some(data),
        }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            code: 200,
            message: msg.into(),
            data: None,
        }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            code: 500,
            message: msg.into(),
            data: None,
        }
    }
}
