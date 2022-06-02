use std::sync::Arc;

use crate::{
    models::{LeaderboardEntry, LeaderboardScrape},
    schema::{associated_leaderboard, leaderboard, leaderboard_scrape, steam_association},
    Error, Result,
};
use chrono::{DateTime, Utc};
use diesel::{
    r2d2::ConnectionManager, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgConnection,
    QueryDsl, RunQueryDsl,
};
use r2d2::Pool;
use serde::Serialize;
use tokio::sync::{oneshot, Semaphore};

type PgConnectionManager = ConnectionManager<PgConnection>;
type PgPool = Pool<PgConnectionManager>;

pub fn make_database_pool() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(Error::from)?;
    let manager: PgConnectionManager = diesel::r2d2::ConnectionManager::new(url);

    diesel::r2d2::Pool::new(manager).map_err(Error::from)
}

#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
    semaphore: Arc<Semaphore>,
}

impl DatabaseService {
    pub fn new() -> Result<Self> {
        let pool = make_database_pool()?;
        let semaphore = Arc::new(Semaphore::new(pool.max_size() as usize));

        Ok(Self { pool, semaphore })
    }

    pub async fn get_leaderboard(&self) -> Result<Leaderboard> {
        let _permit = self.semaphore.acquire().await.map_err(Error::from)?;
        let connection = self.pool.get().map_err(Error::from)?;
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let scrape = leaderboard_scrape::table
                .order_by(leaderboard_scrape::at.desc())
                .offset(1)
                .first::<LeaderboardScrape>(&connection)
                .map_err(Error::from)?;

            leaderboard::table
                .left_join(
                    associated_leaderboard::table.inner_join(steam_association::table.on(
                        associated_leaderboard::steam_association_id.eq(steam_association::id),
                    )),
                )
                .filter(leaderboard::leaderboard_scrape_id.eq(scrape.id))
                .select((
                    leaderboard::rank,
                    leaderboard::avatar,
                    leaderboard::name,
                    leaderboard::rating,
                    leaderboard::wins,
                    leaderboard::losses,
                    steam_association::steam_id.nullable(),
                ))
                .order(leaderboard::rank)
                .load(&connection)
                .map(move |entries| {
                    let timestamp = DateTime::<Utc>::from(scrape.at)
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

                    tx.send(Leaderboard { timestamp, entries }).ok();
                })
                .map_err(Error::from)
        });

        rx.await.map_err(Error::from)
    }
}

#[derive(Debug, Serialize)]
pub struct Leaderboard {
    pub timestamp: String,
    pub entries: Vec<LeaderboardEntry>,
}
