use {
    crate::api_error::ApiError, aide::OperationInput, axum::extract::FromRequest,
    schemars::JsonSchema, serde::de::DeserializeOwned,
};

#[derive(Debug)]
pub struct Path<T>(pub T);

impl<S, T> FromRequest<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(data) = axum::extract::Path::from_request(req, state).await?;
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
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        axum::extract::Path::<T>::inferred_early_responses(ctx, operation)
    }
}
