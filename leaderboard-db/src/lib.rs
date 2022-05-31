#[macro_use]
extern crate diesel;

use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl,
    RunQueryDsl,
};
use models::{NewEntry, NewLeaderboardScrape, NewSteamAssociation};
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

    pub fn hash_avatar_urls(&self) -> Result<usize> {
        let sql = include_str!("hash-avatar-urls.sql");

        diesel::sql_query(sql)
            .execute(&self.connection)
            .map_err(Error::from)
    }

    pub fn get_new_players(&self) -> Result<Vec<(String, String, i32, i32)>> {
        use schema::{avatar_hash, avatar_map, leaderboard, names, steam_association};

        leaderboard::table
            .inner_join(names::table.on(leaderboard::name.eq(names::name)))
            .inner_join(avatar_map::table.on(leaderboard::avatar.eq(avatar_map::url)))
            .inner_join(avatar_hash::table.on(avatar_map::avatar_hash_id.eq(avatar_hash::id)))
            .left_join(
                steam_association::table.on(names::id
                    .eq(steam_association::names_id)
                    .and(avatar_hash::id.eq(steam_association::avatar_hash_id))),
            )
            .filter(steam_association::id.is_null())
            .select((
                leaderboard::name,
                avatar_hash::hash,
                names::id,
                avatar_hash::id,
            ))
            .distinct()
            .load(&self.connection)
            .map_err(Error::from)
    }

    pub fn associate_player(
        &self,
        names_id: i32,
        avatar_hash_id: i32,
        steam_id: Vec<u8>,
    ) -> Result<usize> {
        let record = NewSteamAssociation {
            names_id,
            avatar_hash_id,
            steam_id,
        };

        diesel::insert_into(schema::steam_association::table)
            .values(&record)
            .execute(&self.connection)
            .map_err(Error::from)
    }

    pub fn associate_leaderboard(&self) -> Result<usize> {
        let sql = include_str!("associate-leaderboard.sql");

        diesel::sql_query(sql)
            .execute(&self.connection)
            .map_err(Error::from)
    }
}
