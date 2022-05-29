use super::schema::*;
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

#[derive(Associations, Queryable)]
#[belongs_to(LeaderboardScrape)]
#[table_name = "leaderboard"]
pub struct Leaderboard {
    pub id: i32,
    pub leaderboard_scrape_id: i32,
    pub rank: i32,
    pub avatar: String,
    pub name: String,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Associations, Insertable)]
#[belongs_to(LeaderboardScrape)]
#[table_name = "leaderboard"]
pub struct NewEntry<'a> {
    pub leaderboard_scrape_id: i32,
    pub rank: i32,
    pub avatar: &'a str,
    pub name: &'a str,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Queryable)]
pub struct Names {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "names"]
pub struct NewName {
    pub name: String,
}

#[derive(Queryable)]
pub struct AvatarHash {
    pub id: i32,
    pub hash: String,
}

#[derive(Insertable)]
#[table_name = "avatar_hash"]
pub struct NewAvatarHash {
    pub hash: String,
}

#[derive(Associations, Insertable, Queryable)]
#[belongs_to(AvatarHash)]
#[table_name = "avatar_map"]
pub struct AvatarMap {
    pub url: String,
    pub avatar_hash_id: i32,
}

#[derive(Associations, Queryable)]
#[belongs_to(AvatarHash)]
#[belongs_to(Names)]
#[table_name = "steam_association"]
pub struct SteamAssociation {
    pub id: i32,
    pub names_id: i32,
    pub avatar_hash_id: i32,
    pub steam_id: Vec<u8>,
}

#[derive(Insertable)]
#[table_name = "steam_association"]
pub struct NewSteamAssociation {
    pub names_id: i32,
    pub avatar_hash_id: i32,
    pub steam_id: Vec<u8>,
}

#[derive(Associations, Insertable, Queryable)]
#[belongs_to(SteamAssociation)]
#[belongs_to(Leaderboard)]
#[table_name = "associated_leaderboard"]
pub struct AssociatedLeaderboard {
    pub steam_association_id: i32,
    pub leaderboard_id: i32,
}
