use super::schema::{leaderboard, leaderboard_scrape};
use std::time::SystemTime;

#[derive(Queryable)]
pub struct LeaderboardScrape {
    pub id: i32,
    pub at: SystemTime,
}

#[derive(Insertable)]
#[table_name = "leaderboard_scrape"]
pub struct NewLeaderboardScrape {
    pub at: SystemTime,
}

#[derive(Queryable)]
pub struct Leaderboard<'a> {
    pub id: i32,
    pub scrape_id: i32,
    pub rank: i32,
    pub avatar: &'a str,
    pub name: &'a str,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Insertable)]
#[table_name = "leaderboard"]
pub struct NewEntry<'a> {
    pub scrape_id: i32,
    pub rank: i32,
    pub avatar: &'a str,
    pub name: &'a str,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}
