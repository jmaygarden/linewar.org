use crate::cache::CacheService;
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
        .layer(TraceLayer::new_for_http())
        .layer(Extension(service));
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
    cache: CacheService,
    db: DatabaseService,
}

impl Services {
    async fn start() -> Self {
        let db = DatabaseService::new().expect("error connecting to database");
        let cache = CacheService::new()
            .await
            .expect("error connecting to cache");

        Self { cache, db }
    }
}

#[tracing::instrument(skip(services))]
#[axum_macros::debug_handler]
async fn root(
    Extension(services): Extension<Services>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response = services
        .cache
        .get_cached("root", || {
            Box::pin(async move {
                let context = services.db.get_leaderboard().await?;
                let template = RootTemplate { context };
                let response = template.render()?;

                Ok(response)
            })
        })
        .await
        .map_err(into_error_response)?;

    Ok(Html(response))
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response = services
        .cache
        .get_cached(format!("player/{steam_id}").as_str(), || {
            Box::pin(async move {
                let context = services.db.get_player(steam_id).await?;
                let template = PlayerTemplate { context };
                let response = template.render()?;

                Ok(response)
            })
        })
        .await
        .map_err(into_error_response)?;

    Ok(Html(response))
}

#[derive(Template)]
#[template(path = "player.html")]
struct PlayerTemplate {
    context: Player,
}

fn into_error_response<E>(error: E) -> (StatusCode, String)
where
    E: std::fmt::Debug,
{
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {error:?}"),
    )
}

mod cache;
