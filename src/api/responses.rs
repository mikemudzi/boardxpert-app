use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::optimizer::OptimizeResult;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct OptimizeResponse {
    pub job_reference: String,
    #[serde(flatten)]
    pub result: OptimizeResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_base64: Option<String>,
}

/// Response when an async job is created
#[derive(Debug, Serialize, ToSchema)]
pub struct AsyncJobResponse {
    pub job_id: Uuid,
    pub status: String,
}

/// Response for job status polling
#[derive(Debug, Serialize, ToSchema)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub job_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<OptimizeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}
