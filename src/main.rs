mod auth;
mod http_request;

use axum::body::Body;
use axum::http::Request;
use axum::{Json, Router};
use clap::Parser;
use serde::{
    Deserialize,
    Serialize
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

#[derive(clap::Parser, Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    #[clap(short, long, default_value = "0.0.0.0:1920")]
    pub listen: SocketAddr,

    #[clap(long, default_value = "account")]
    pub audience: String,

    #[clap(long, default_value = "https://keycloak.localhost/realms/master/.well-known/openid-configuration")]
    pub oidc: ::url::Url,
}

#[derive(Debug, Clone)]
pub struct ApplicationState {
    pub args: Arc<Args>,
    // pub pg_pool: PgPool,
}

pub type State = axum::extract::State<ApplicationState>;

#[tokio::main]
pub async fn main() {
    env_logger::init();

    // Lese CLI Argumente
    let args = Args::parse();
    let listen = args.listen.clone();

    let state = ApplicationState {
        // pg_pool: sqlx::postgres::PgPoolOptions::new()
        //     .connect(args.db_url.as_str())
        //     .await
        //     .expect("Failed to connect to database"),
        args: Arc::new(args),
    };

    let app = NormalizePathLayer::trim_trailing_slash()
        .layer(Router::new()

            // Middleware sind zusätzliche schritte die vor der Business-logic stattfinden und bereiten sozusagen die Anfrage für die entgültige verarbeitung vor.
            .layer(axum::middleware::from_fn_with_state(state.clone(), auth::auth_jwt))
            .route("/health", axum::routing::get(async || Json(serde_json::json! {{
                "status": "healthy",
            }})))
            .with_state(state.clone())
        .layer(axum::middleware::from_fn(log_request)));

    let server = tokio::net::TcpListener::bind(&listen).await
        .expect("Failed to bind");

    log::debug!("Binding to {:?}", listen);

    axum::serve(server, axum::ServiceExt::<Request<Body>>::into_make_service(app)).await
        .expect("Application service encountered an error");
}

pub(crate) const LOG_TARGET: &str = "api::routes";

/// Logging middleware stellt sicher, dass Anfragen in der konsole erscheinen. Falls wir mal einen verhalten-basiertes EPP haben, ist das sehr nützlich.
async fn log_request(req: Request<Body>, next: axum::middleware::Next) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Continue to the next middleware or handler
    let res = next.run(req).await;

    let response = res.status();

    // Log the request details
    match response {
        response if response.is_success() => log::info!(target: LOG_TARGET, "Incoming request: {} {} {}", method, response, uri),
        response if response.is_informational() || response.is_redirection() => log::debug!(target: LOG_TARGET, "Incoming request: {} {} {}", method, response, uri),
        response if response.is_client_error() => log::warn!(target: LOG_TARGET, "Incoming request: {} {} {}", method, response, uri),
        response if response.is_server_error() => log::error!(target: LOG_TARGET, "Incoming request: {} {} {}", method, response, uri),
        response => log::trace!(target: LOG_TARGET, "Incoming request: {} {} {}", method, response, uri)
    }

    return res;
}
