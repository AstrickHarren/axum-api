use {
    aide::OperationOutput,
    axum::{
        Json,
        extract::{
            multipart::MultipartError,
            rejection::{JsonRejection, PathRejection, QueryRejection},
        },
        http::StatusCode,
        response::IntoResponse,
    },
    axum_typed_multipart::TypedMultipartError,
    schemars::JsonSchema,
    serde::Serialize,
    serde_json::json,
    serde_with::{FromInto, serde_as},
    validator::ValidationErrors,
};

#[serde_as]
#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiError {
    #[serde_as(as = "FromInto<u16>")]
    pub status: StatusCode,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl Default for ApiError {
    fn default() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: Default::default(),
            detail: Default::default(),
            extensions: Default::default(),
        }
    }
}

pub trait IntoApiError {
    fn into_error_response(self) -> ApiError;
}

impl<T> From<T> for ApiError
where
    T: IntoApiError,
{
    fn from(value: T) -> Self {
        value.into_error_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self)).into_response()
    }
}

impl OperationOutput for ApiError {
    type Inner = Json<Self>;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        Self::Inner::operation_response(ctx, operation)
    }
}

impl IntoApiError for JsonRejection {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: self.status(),
            title: "Axum Json Rejection".to_string(),
            detail: Some(self.body_text()),
            extensions: None,
        }
    }
}

impl IntoApiError for ValidationErrors {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            title: "Validation error".to_string(),
            detail: None,
            extensions: Some(json!({
                "validations": self
            })),
        }
    }
}

impl IntoApiError for QueryRejection {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: self.status(),
            title: "Axum Query Rejection".to_string(),
            detail: Some(self.body_text()),
            extensions: None,
        }
    }
}

impl IntoApiError for PathRejection {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: self.status(),
            title: "Axum Path Rejection".to_string(),
            detail: Some(self.body_text()),
            extensions: None,
        }
    }
}

impl IntoApiError for TypedMultipartError {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: self.get_status(),
            title: "Multipart Parse Rejection".to_string(),
            detail: Some(self.to_string()),
            extensions: None,
        }
    }
}

impl IntoApiError for MultipartError {
    fn into_error_response(self) -> ApiError {
        ApiError {
            status: self.status(),
            title: "Multipart Parse Rejection".to_string(),
            detail: Some(self.to_string()),
            extensions: None,
        }
    }
}
