use {
    crate::api_error::ApiError,
    aide::{OperationInput, openapi::StatusCode},
    axum::extract::FromRequestParts,
    schemars::JsonSchema,
    serde::de::DeserializeOwned,
};

#[derive(Debug)]
pub struct Path<T>(pub T);

impl<S, T> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(data) =
            axum::extract::Path::from_request_parts(parts, state).await?;
        Ok(Path(data))
    }
}

impl<T: JsonSchema> OperationInput for Path<T> {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        axum::extract::Path::<T>::operation_input(ctx, operation)
    }

    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<StatusCode>, aide::openapi::Response)> {
        axum::extract::Path::<T>::inferred_early_responses(ctx, operation)
    }
}
