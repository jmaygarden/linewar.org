#[macro_use]
extern crate diesel;

use diesel::{Connection, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};
use models::{NewEntry, NewLeaderboardScrape};
use std::{env::VarError, time::SystemTime};

pub mod models;
pub mod schema;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] diesel::result::ConnectionError),
    #[error("Databse query error: {0}")]
    QueryError(#[from] diesel::result::Error),
    #[error("DATABASE_URL must be set")]
    UrlNotSet(#[from] VarError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct LeaderboardDatabase {
    connection: PgConnection,
}

impl LeaderboardDatabase {
    pub fn new() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").map_err(Error::from)?;
        let connection = PgConnection::establish(database_url.as_str()).map_err(Error::from)?;

        Ok(Self { connection })
    }

    pub fn start_scrape(&self) -> Result<models::LeaderboardScrape> {
        let record = NewLeaderboardScrape {
            at: SystemTime::now(),
        };

        let result = diesel::insert_into(schema::leaderboard_scrape::table)
            .values(&record)
            .get_results(&self.connection)
            .map_err(Error::from)?
            .into_iter()
            .nth(0)
            .expect("returned empty vector");

        Ok(result)
    }

    pub fn store_entries(&self, records: &[NewEntry]) -> Result<usize> {
        diesel::insert_into(schema::leaderboard::table)
            .values(records)
            .execute(&self.connection)
            .map_err(Error::from)
    }

    pub fn index_names(&self) -> Result<usize> {
        let new_names = schema::leaderboard::table
            .select(schema::leaderboard::name)
            .distinct()
            .left_join(schema::names::table.on(schema::leaderboard::name.eq(schema::names::name)))
            .filter(schema::names::name.is_null());

        diesel::insert_into(schema::names::table)
            .values(new_names)
            .into_columns(schema::names::name)
            .execute(&self.connection)
            .map_err(Error::from)
    }
}
