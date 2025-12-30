pub mod aide_ext;
pub mod api_error;
mod auth;
pub mod diesel_otel;
pub mod extractors;
pub mod prelude;
mod scalar;

use {
    crate::{diesel_otel::OtelInstrument, extractors::Jwt, scalar::Scalar},
    aide::{
        axum::ApiRouter,
        openapi::{OpenApi, SecurityScheme},
    },
    axum::Extension,
    axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer},
    derive_builder::Builder,
    diesel_async::{AsyncConnection, AsyncMigrationHarness, AsyncPgConnection},
    diesel_migrations::{EmbeddedMigrations, MigrationHarness},
    std::{net::SocketAddr, sync::Arc},
    tokio::net::{TcpListener, ToSocketAddrs},
};
pub use {extractors::jwt_open_api, init_tracing_opentelemetry::TracingConfig};

#[derive(Builder)]
#[builder(pattern = "owned", name = "Config", build_fn(name = "make_server"))]
pub struct Server<A: ToSocketAddrs> {
    app: ApiRouter,
    addr: A,
    /// PostgreSQL connection URL for Diesel
    #[builder(default, setter(into, strip_option))]
    pg_url: Option<String>,
    /// Jwt token secret
    #[builder(setter(into, strip_option))]
    jwt_secret: String,
    /// Dictate `Scalar`'s version, 1.34.2 is a great choice for example.
    #[builder(default, setter(into, strip_option))]
    scalar_version: Option<String>,
    #[builder(default)]
    migratons: Option<EmbeddedMigrations>,
    #[builder(default = TracingConfig::development())]
    otel_config: TracingConfig,
}

impl<A: ToSocketAddrs> Server<A> {
    pub async fn serve(self) -> Result<(), eyre::Error> {
        let _guard = self.otel_config.init_subscriber()?;

        let app = {
            let mut api = OpenApi::default();
            aide::generate::all_error_responses(true);
            let mut app = self
                .app
                .finish_api_with(&mut api, |o| {
                    o.title("Axum Api").security_scheme(
                        "Json Web Token",
                        SecurityScheme::Http {
                            scheme: "Bearer".to_string(),
                            bearer_format: None,
                            description: Some("Bearer token using JWT".to_string()),
                            extensions: Default::default(),
                        },
                    )
                })
                // OTEL
                .layer(OtelInResponseLayer::default())
                .layer(OtelAxumLayer::default().try_extract_client_ip(true))
                // Open API
                .merge(Scalar::new(self.scalar_version).router())
                .layer(Extension(Arc::new(api)))
                // CORS
                .layer(cors_layer())
                // Jwt
                .layer(Extension(Jwt::new(self.jwt_secret.as_bytes())));

            // Diesel
            if let Some(pg_url) = &self.pg_url {
                let database = AsyncPgConnection::establish(pg_url).await?;
                let mut database = AsyncMigrationHarness::new(database);
                if let Some(migrations) = self.migratons {
                    database
                        .run_pending_migrations(migrations)
                        .expect("Migration failed");
                }
                let mut database = database.into_inner();
                database.set_instrumentation(OtelInstrument);
                app = app.layer(Extension(Arc::new(database)))
            };

            app
        };
        let listener = TcpListener::bind(self.addr).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    }
}

fn cors_layer() -> tower_http::cors::CorsLayer {
    use tower_http::cors::CorsLayer;
    #[cfg(debug_assertions)]
    {
        CorsLayer::very_permissive()
    }
    #[cfg(not(debug_assertions))]
    {
        CorsLayer::new()
    }
}
