use dotenv::dotenv;
use leaderboard_db::service::DatabaseService;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = DatabaseService::new().expect("error connection to database");
    let leaderboard = db
        .get_leaderboard()
        .await
        .expect("error retrieving leaderboard");
    let json = serde_json::to_string_pretty(&leaderboard).expect("error serializing leaderboard");

    println!("{json}");
}
