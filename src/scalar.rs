use {
    aide::openapi::OpenApi,
    axum::{Extension, Json, Router, handler::Handler, response::Html, routing::get},
    std::sync::Arc,
};
pub struct Scalar {
    path: String,
    version: Option<String>,
}

impl Scalar {
    pub fn new(version: Option<String>) -> Self {
        Self {
            path: "/private/api.json".to_string(),
            version,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/docs", get(self.axum_handler()))
            .route(
                &self.path,
                get(|Extension(api): Extension<Arc<OpenApi>>| async { Json(api) }),
            )
    }

    fn axum_handler(&self) -> impl Handler<((),), ()> {
        let html = format!(
            r#"
            <!doctype html>
            <html>
              <head>
                <title>Scalar API Reference</title>
                <meta charset="utf-8" />
                <meta
                  name="viewport"
                  content="width=device-width, initial-scale=1" />
              </head>

              <body>
                <div id="app"></div>

                <!-- Load the Script -->
                <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference{}"></script>

                <!-- Initialize the Scalar API Reference -->
                <script>
                  Scalar.createApiReference('#app', {{
                    // The URL of the OpenAPI/Swagger document
                    url: '{}',
                    persistAuth: true,
                    authentication: {{
                        preferredSecurityScheme: 'Json Web Token'
                    }},
                  }})
                </script>
              </body>
            </html>
            "#,
            self.version
                .as_ref()
                .map(|v| format!("@{}", v))
                .unwrap_or_default(),
            self.path,
        );

        || async { Html(html) }
    }
}
