use {
    crate::prelude::ApiError,
    aide::{
        OperationInput,
        openapi::{MediaType, RequestBody, SchemaObject, StatusCode},
        operation::set_body,
    },
    axum::extract::FromRequest,
    axum_typed_multipart::TryFromMultipartWithState,
    derive_more::{AsMut, AsRef, Deref, DerefMut, From},
    indexmap::IndexMap,
    schemars::JsonSchema,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Deref, DerefMut, AsRef, AsMut, From)]
pub struct TypedMultipart<T>(pub T);

impl<S, T> FromRequest<S> for TypedMultipart<T>
where
    S: Send + Sync,
    T: TryFromMultipartWithState<S>,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            axum_typed_multipart::TypedMultipart::from_request(req, state)
                .await?
                .0,
        ))
    }
}

impl<T: JsonSchema> OperationInput for TypedMultipart<T> {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        let json_schema = ctx.schema.subschema_for::<T>();
        set_body(
            ctx,
            operation,
            RequestBody {
                description: Some("multipart form data".into()),
                content: IndexMap::from_iter([(
                    "multipart/form-data".into(),
                    MediaType {
                        schema: Some(SchemaObject {
                            json_schema,
                            external_docs: None,
                            example: None,
                        }),
                        ..Default::default()
                    },
                )]),
                required: true,
                extensions: IndexMap::default(),
            },
        );
    }

    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<StatusCode>, aide::openapi::Response)> {
        axum::extract::Multipart::inferred_early_responses(ctx, operation)
    }
}
