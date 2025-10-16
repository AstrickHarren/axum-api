use {
    crate::prelude::ApiError,
    aide::{
        OperationInput,
        openapi::{MediaType, RequestBody, SchemaObject},
        operation::set_body,
    },
    axum::extract::FromRequest,
    axum_typed_multipart::{TryFromMultipartWithState, TypedMultipart},
    derive_more::{AsMut, AsRef, Deref, DerefMut, From},
    indexmap::IndexMap,
    schemars::{JsonSchema, json_schema},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Deref, DerefMut, AsRef, AsMut, From)]
pub struct Multipart<T>(pub T);

impl<S, T> FromRequest<S> for Multipart<T>
where
    S: Send + Sync,
    T: TryFromMultipartWithState<S>,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(TypedMultipart::from_request(req, state).await?.0))
    }
}

impl<T: JsonSchema> OperationInput for Multipart<T> {
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
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        axum::extract::Multipart::inferred_early_responses(ctx, operation)
    }
}
