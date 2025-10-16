use std::error::Error;

use aide::axum::{ApiRouter, routing::get_with};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = ApiRouter::new().api_route(
        "/health",
        get_with(|| async {}, |o| o.summary("Health Check")),
    );
    axum_api::Config::default()
        .app(app)
        .pg_url("postgres://postgres:password@localhost/postgres")
        .jwt_secret("my_jwt_secret")
        .addr("127.0.0.1:8080")
        .make_server()?
        .serve()
        .await?;
    Ok(())
}
