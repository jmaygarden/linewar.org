use crate::cache::CacheService;
use askama::Template;
use axum::{
    extract::Path,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use leaderboard_db::service::{DatabaseService, Leaderboard, Player};
use std::net::SocketAddr;
use tokio::sync::oneshot;
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
        .route("/plot/rating/:steam_id", get(plot_rating))
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

#[tracing::instrument(skip(services))]
async fn plot_rating(
    Extension(services): Extension<Services>,
    Path(steam_id): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response = services
        .cache
        .get_cached(format!("plot/rating/{steam_id}").as_str(), || {
            Box::pin(async move {
                use plotters::prelude::*;

                let context = services.db.get_player(steam_id).await?;
                let (tx, rx) = oneshot::channel();

                tokio::task::spawn_blocking(move || {
                    let mut svg = String::new();
                    {
                        let root =
                            SVGBackend::with_string(&mut svg, (640, 480)).into_drawing_area();

                        root.fill(&WHITE).unwrap();

                        let x_min = context.history.iter().map(|x| x.timestamp).min().unwrap();
                        let x_max = context.history.iter().map(|x| x.timestamp).max().unwrap();
                        let mut chart = ChartBuilder::on(&root)
                            .caption(
                                format!("{} Rating", context.player.name),
                                ("sans-serif", 30).into_font(),
                            )
                            .margin(20)
                            .x_label_area_size(30)
                            .y_label_area_size(30)
                            .build_cartesian_2d(x_min..x_max, 0f32..50f32)
                            .unwrap();

                        chart
                            .configure_mesh()
                            .x_label_formatter(&|x| format!("{}", x.date().naive_utc()))
                            .draw()
                            .unwrap();
                        chart
                            .draw_series(
                                AreaSeries::new(
                                    context.history.iter().map(|i| (i.timestamp, i.rating)),
                                    0f32,
                                    &BLUE.mix(0.2),
                                )
                                .border_style(&BLUE),
                            )
                            .unwrap();
                    }

                    tx.send(svg).ok();
                });

                let response = rx.await?;

                Ok(response)
            })
        })
        .await
        .map_err(into_error_response)?;

    Ok(([(CONTENT_TYPE, "image/svg+xml")], response))
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
