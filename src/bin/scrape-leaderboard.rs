use dotenv::dotenv;
use leaderboard_db::{models::{self, NewEntry}, LeaderboardDatabase};
use leaderboard_scraper::scrape_leaderboard;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = LeaderboardDatabase::new().expect("error connecting to databse");
    let scrape = db
        .start_scrape()
        .expect("failed to create new scrape entry");
    let leaderboard = scrape_leaderboard()
        .await
        .expect("Error fetching the Line War leaderboard.");
    let records: Vec<NewEntry> = leaderboard.iter()
        .map(|entry| models::NewEntry {
            leaderboard_scrape_id: scrape.id,
            rank: entry.rank,
            avatar: entry.avatar.as_str(),
            name: entry.name.as_str(),
            rating: entry.rating,
            wins: entry.wins,
            losses: entry.losses,
        })
        .collect();
    let n = db.store_entries(records.as_slice()).expect("failed to store entry");

    println!("Wrote {n} records at {:?}.", scrape.at);
}
