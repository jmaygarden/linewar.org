use askama::Template;
use axum::{extract::Path, http::StatusCode, response::Html, routing::get, Extension, Router};
use leaderboard_db::service::{DatabaseService, Leaderboard, Player};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let service = Services::start().await;
    let app = Router::new()
        .route("/", get(root))
        .route("/player/:steam_id", get(player))
        .layer(Extension(service));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("critical error");
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

async fn root(Extension(services): Extension<Services>) -> Result<Html<String>, StatusCode> {
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

async fn player(
    Extension(services): Extension<Services>,
    Path(steam_id): Path<u64>,
) -> Result<Html<String>, StatusCode> {
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
