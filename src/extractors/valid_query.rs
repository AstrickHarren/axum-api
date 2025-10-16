use {
    crate::api_error::ApiError, aide::OperationInput, axum::extract::FromRequest,
    schemars::JsonSchema, serde::de::DeserializeOwned, validator::Validate,
};

pub struct Query<T>(pub T);

impl<S, T> FromRequest<S> for Query<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(data) =
            axum::extract::Query::<T>::from_request(req, state).await?;
        data.validate()?;
        Ok(Query(data))
    }
}

impl<T: JsonSchema> OperationInput for Query<T> {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        axum::extract::Query::<T>::operation_input(ctx, operation)
    }

    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        axum::extract::Query::<T>::inferred_early_responses(ctx, operation)
    }
}
