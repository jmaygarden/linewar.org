use crate::{
    models::{LeaderboardEntry, LeaderboardScrape, PlayerIGN, PlayerStatistics, RecentLeaderboard},
    schema::{
        associated_leaderboard, avatar_hash, current_leaderboard, leaderboard, leaderboard_scrape,
        names, steam_association,
    },
    Error, Result,
};
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use diesel::{
    r2d2::ConnectionManager, ExpressionMethods, NullableExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl,
};
use r2d2::{Pool, PooledConnection};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{oneshot, Semaphore, SemaphorePermit};

type PgConnectionManager = ConnectionManager<PgConnection>;
type PgPooledConnection = PooledConnection<PgConnectionManager>;
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

    async fn setup_request<'a, T>(
        &'a self,
    ) -> Result<(DatabaseContext<'a, T>, oneshot::Receiver<T>)> {
        let _permit = self.semaphore.acquire().await.map_err(Error::from)?;
        let connection = self.pool.get().map_err(Error::from)?;
        let (tx, rx) = oneshot::channel();

        Ok((
            DatabaseContext {
                _permit,
                connection,
                tx,
            },
            rx,
        ))
    }

    pub async fn get_leaderboard(&self) -> Result<Leaderboard> {
        let (context, rx) = self.setup_request().await?;
        let LeaderboardScrape { at, .. } = get_latest_scrape(&context.connection)?;

        tokio::task::spawn_blocking(move || {
            let timestamp = at.into();
            let result = current_leaderboard::table
                .select((
                    current_leaderboard::rank,
                    current_leaderboard::avatar,
                    current_leaderboard::name,
                    current_leaderboard::rating,
                    current_leaderboard::wins,
                    current_leaderboard::losses,
                    current_leaderboard::steam_id.nullable(),
                ))
                .load(&context.connection)
                .map(|entries| Leaderboard { timestamp, entries })
                .map_err(Error::from);

            context.tx.send(result).ok();
        });

        rx.await
            .map_err(Error::from)
            .and_then(std::convert::identity)
    }

    pub async fn get_player(&self, steam_id: u64) -> Result<Player> {
        let (context, rx) = self.setup_request().await?;
        let steam_id_bytes = steam_id.to_le_bytes();

        tokio::task::spawn_blocking(move || {
            let scrape = get_latest_scrape(&context.connection)?;
            let (steam_association_id, player) = steam_association::table
                .inner_join(names::table)
                .inner_join(avatar_hash::table)
                .select((steam_association::id, (names::name, avatar_hash::hash)))
                .filter(steam_association::steam_id.eq(steam_id_bytes.as_slice()))
                .first::<(i32, PlayerIGN)>(&context.connection)
                .map_err(Error::from)?;

            associated_leaderboard::table
                .inner_join(leaderboard::table.inner_join(leaderboard_scrape::table))
                .filter(associated_leaderboard::steam_association_id.eq(steam_association_id))
                .select((
                    leaderboard_scrape::at,
                    leaderboard::rank,
                    leaderboard::rating,
                    leaderboard::wins,
                    leaderboard::losses,
                ))
                .order(leaderboard_scrape::at.desc())
                .load::<PlayerStatistics>(&context.connection)
                .map(|player_rating| {
                    let timestamp = scrape.at.into();
                    let player = PlayerId {
                        name: player.name,
                        avatar: player.avatar_url,
                        steam_id,
                    };
                    let history = player_rating.into_iter().map(History::from).collect();

                    context.tx.send(Player {
                        timestamp,
                        player,
                        history,
                    })
                })
                .map_err(Error::from)
        });

        rx.await.map_err(Error::from)
    }

    pub async fn get_recent_leaderboard(&self) -> Result<Vec<RecentLeaderboard>> {
        let (context, rx) = self.setup_request().await?;

        tokio::task::spawn_blocking(move || {
            let sql = include_str!("latest-leaderboard.sql");

            diesel::sql_query(sql)
                .load::<RecentLeaderboard>(&context.connection)
                .map(move |entries| context.tx.send(entries).ok())
                .map_err(Error::from)
        });

        rx.await.map_err(Error::from)
    }
}

struct DatabaseContext<'a, T> {
    _permit: SemaphorePermit<'a>,
    connection: PgPooledConnection,
    tx: oneshot::Sender<T>,
}

fn get_latest_scrape(connection: &PgPooledConnection) -> Result<LeaderboardScrape> {
    leaderboard_scrape::table
        .order_by(leaderboard_scrape::at.desc())
        .offset(1)
        .first(connection)
        .map_err(Error::from)
}

#[derive(Debug, Serialize)]
pub struct Leaderboard {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Debug, Serialize)]
pub struct Player {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub player: PlayerId,
    pub history: Vec<History>,
}

#[derive(Debug, Serialize)]
pub struct PlayerId {
    pub name: String,
    pub avatar: String,
    pub steam_id: u64,
}

#[derive(Debug, Serialize)]
pub struct History {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub rank: i32,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}

impl From<PlayerStatistics> for History {
    fn from(value: PlayerStatistics) -> Self {
        let PlayerStatistics {
            timestamp,
            rank,
            rating,
            wins,
            losses,
        } = value;
        let timestamp = timestamp.into();

        Self {
            timestamp,
            rank,
            rating,
            wins,
            losses,
        }
    }
}
