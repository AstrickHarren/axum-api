use {
  crate::api_error::ApiError,
  aide::{OperationInput, OperationOutput},
  axum::{extract::FromRequest, response::IntoResponse},
  derive_more::{AsMut, AsRef, Deref, DerefMut, From},
  diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    serialize::ToSql,
    sql_types,
  },
  schemars::JsonSchema,
  serde::{Deserialize, Serialize, de::DeserializeOwned},
  validator::Validate,
};

#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  Deref,
  DerefMut,
  From,
  Serialize,
  Deserialize,
  AsRef,
  AsMut,
  PartialEq,
  FromSqlRow,
  AsExpression,
)]
#[serde(transparent)]
#[diesel(sql_type = sql_types::Jsonb)]
pub struct Json<T>(pub T);

impl<S, T> FromRequest<S> for Json<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ApiError;

  async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
    let axum::Json(data) = axum::Json::<T>::from_request(req, state).await?;
    data.validate()?;
    Ok(Json(data))
  }
}

impl<T: Serialize> IntoResponse for Json<T> {
  fn into_response(self) -> axum::response::Response {
    axum::Json(self.0).into_response()
  }
}

impl<T: JsonSchema> OperationInput for Json<T> {
  fn operation_input(
    ctx: &mut aide::generate::GenContext,
    operation: &mut aide::openapi::Operation,
  ) {
    <axum::Json<T> as OperationInput>::operation_input(ctx, operation)
  }

  fn inferred_early_responses(
    ctx: &mut aide::generate::GenContext,
    operation: &mut aide::openapi::Operation,
  ) -> Vec<(Option<u16>, aide::openapi::Response)> {
    <axum::Json<T> as OperationInput>::inferred_early_responses(ctx, operation)
  }
}
impl<T: JsonSchema> OperationOutput for Json<T> {
  type Inner = <axum::Json<T> as OperationOutput>::Inner;

  fn operation_response(
    ctx: &mut aide::generate::GenContext,
    operation: &mut aide::openapi::Operation,
  ) -> Option<aide::openapi::Response> {
    <axum::Json<T> as OperationOutput>::operation_response(ctx, operation)
  }

  fn inferred_responses(
    ctx: &mut aide::generate::GenContext,
    operation: &mut aide::openapi::Operation,
  ) -> Vec<(Option<u16>, aide::openapi::Response)> {
    <axum::Json<T> as OperationOutput>::inferred_responses(ctx, operation)
  }
}

impl<T> FromSql<sql_types::Jsonb, Pg> for Json<T>
where
  T: std::fmt::Debug + DeserializeOwned,
{
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<sql_types::Jsonb, Pg>>::from_sql(bytes)?;
    Ok(Json(serde_json::from_value::<T>(value)?))
  }
}

impl<T> ToSql<sql_types::Jsonb, Pg> for Json<T>
where
  T: std::fmt::Debug + Serialize,
{
  fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<sql_types::Jsonb, Pg>>::to_sql(&value, &mut out.reborrow())
  }
}
