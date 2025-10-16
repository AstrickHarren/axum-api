mod jwt;
mod path;
mod valid_json;
mod valid_query;

use {
    aide::OperationInput, axum::extract::FromRequestParts, diesel_async::AsyncPgConnection,
    std::sync::Arc,
};
pub use {
    jwt::{Claims, Jwt, jwt_open_api},
    path::Path,
    valid_json::Json,
    valid_query::Query,
};

pub struct Database(Arc<AsyncPgConnection>);
impl Database {
    pub fn conn(&self) -> &AsyncPgConnection {
        self.0.as_ref()
    }
}
impl OperationInput for Database {}
impl<S: Sync> FromRequestParts<S> for Database {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let db = parts.extensions.get::<Arc<AsyncPgConnection>>().ok_or(())?;
        Ok(Database(db.clone()))
    }
}
