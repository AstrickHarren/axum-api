use {
    crate::{api_error::ApiError, extractors::Json},
    aide::{
        OperationInput,
        transform::{TransformOperation, TransformPathItem},
    },
    axum::{
        extract::FromRequestParts,
        http::{self, StatusCode},
    },
    chrono::{DateTime, Duration, Utc, serde::ts_seconds},
    derive_more::{Deref, DerefMut},
    jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode},
    schemars::JsonSchema,
    serde::{Deserialize, Serialize, de::DeserializeOwned},
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct Claims<T> {
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
    #[deref]
    #[deref_mut]
    #[serde(flatten)]
    pub inner: T,
}

impl<T> OperationInput for Claims<T> {}
impl<S: Sync, T: DeserializeOwned> FromRequestParts<S> for Claims<T> {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jwt = Jwt::from_request_parts(parts, state).await?;
        let token = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "))
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .ok_or_else(|| ApiError {
                status: StatusCode::UNAUTHORIZED,
                title: "Missing Token".to_string(),
                ..Default::default()
            })?;
        jwt.decode(token)
    }
}

#[derive(Clone)]
pub struct Jwt {
    keys: Arc<JwtKey>,
}

impl OperationInput for Jwt {}
impl<S: Sync> FromRequestParts<S> for Jwt {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let jwt = parts
            .extensions
            .get::<Jwt>()
            .expect("No jwt keys found, perhaps forgot to set it in extension")
            .clone();
        Ok(jwt)
    }
}

impl Jwt {
    pub fn encode<T: Serialize>(&self, data: T, expiration: Duration) -> Result<String, ApiError> {
        let iat = Utc::now();
        let exp = iat.checked_add_signed(expiration).unwrap();
        let claims = Claims {
            iat,
            exp,
            inner: data,
        };
        let jwt = encode(&Header::default(), &claims, &self.keys.enc).map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Token Creation Failure".to_string(),
            detail: Some(e.to_string()),
            extensions: None,
        })?;
        Ok(jwt)
    }

    pub(crate) fn new(secret: &[u8]) -> Self {
        Self {
            keys: Arc::new(JwtKey::new(secret)),
        }
    }

    pub fn decode<T: DeserializeOwned>(&self, token: &str) -> Result<Claims<T>, ApiError> {
        let token_data = self.decode_raw::<Claims<T>>(token).map_err(|err| {
            tracing::error!("Error decoding token: {:?}", err);
            ApiError {
                status: StatusCode::UNAUTHORIZED,
                title: "Invalid Token".to_string(),
                ..Default::default()
            }
        })?;
        if Utc::now() > token_data.claims.exp {
            Err(ApiError {
                status: StatusCode::UNAUTHORIZED,
                title: "Token Expired".to_string(),
                ..Default::default()
            })?;
        }
        Ok(token_data.claims)
    }

    fn decode_raw<T: DeserializeOwned>(
        &self,
        token: &str,
    ) -> Result<TokenData<T>, jsonwebtoken::errors::Error> {
        decode::<T>(token, &self.keys.dec, &Validation::default())
    }
}

pub struct JwtKey {
    pub enc: EncodingKey,
    pub dec: DecodingKey,
}
impl JwtKey {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            enc: EncodingKey::from_secret(secret),
            dec: DecodingKey::from_secret(secret),
        }
    }
}

// Only for OpenAPI purpose
#[derive(Debug, JsonSchema)]
#[allow(dead_code)]
pub struct JwtUnauthErr {
    #[schemars(example = 401)]
    status: u16,
    #[schemars(example = "Invalid Token".to_string())]
    title: String,
}
pub fn jwt_open_api(o: TransformPathItem) -> TransformPathItem {
    let mut o = o.security_requirement("Json Web Token");
    aide::util::iter_operations_mut(o.inner_mut())
        .map(|(_, o)| TransformOperation::new(o))
        .for_each(|o| {
            let _ =
                o.response_with::<401, Json<JwtUnauthErr>, _>(|r| r.description("Unauthorized"));
        });
    o
}
