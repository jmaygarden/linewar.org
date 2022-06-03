use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use leaderboard_db::service::{DatabaseService, Leaderboard, Player};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() {
    setup();

    let service = Services::start().await;
    let app = Router::new()
        .route("/", get(root))
        .route("/player/:steam_id", get(player))
        .layer(Extension(service))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("critical error");
}

fn setup() {
    dotenv::dotenv().ok();

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();
}

#[derive(Clone)]
struct Services {
    db: DatabaseService,
}

impl Services {
    async fn start() -> Self {
        let db = DatabaseService::new().expect("error connecting to database");

        Self { db }
    }
}

#[tracing::instrument(skip(services))]
async fn root(Extension(services): Extension<Services>) -> impl IntoResponse {
    let context = services
        .db
        .get_leaderboard()
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = RootTemplate { context };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_error) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "root.html")]
struct RootTemplate {
    context: Leaderboard,
}

#[tracing::instrument(skip(services))]
async fn player(
    Extension(services): Extension<Services>,
    Path(steam_id): Path<u64>,
) -> impl IntoResponse {
    let context = services
        .db
        .get_player(steam_id)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = PlayerTemplate { context };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_error) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "player.html")]
struct PlayerTemplate {
    context: Player,
}