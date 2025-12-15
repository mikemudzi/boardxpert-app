use serde::Serialize;
use crate::optimizer::OptimizeResult;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(result: T) -> Self {
        ApiResponse {
            success: true,
            result: Some(result),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn error(code: &str, message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            result: None,
            error: Some(ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                field: None,
            }),
        }
    }

    pub fn error_with_field(code: &str, message: &str, field: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            result: None,
            error: Some(ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                field: Some(field.to_string()),
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OptimizeResponse {
    pub job_reference: String,
    #[serde(flatten)]
    pub result: OptimizeResult,
}
