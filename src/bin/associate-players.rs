use dotenv::dotenv;
use leaderboard_db::{LeaderboardDatabase, schema};
use leaderboard_scraper::steam::Steam;

#[tokio::main]
async fn main() {
    let (db, steam) = init().await;
    let n = db.index_names().expect("error indexing new names");
    println!("Indexed {n} new names.")
    //db.hash_avatar_urls().expect("error hashing new avatar URLs");
}

async fn init() -> (LeaderboardDatabase, Steam) {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = LeaderboardDatabase::new().expect("error connecting to databse");
    let mut steam = Steam::new().expect("error initializing Steam client");

    steam
        .start_session()
        .await
        .expect("failed to fetch the session ID cookie");

    (db, steam)
}
