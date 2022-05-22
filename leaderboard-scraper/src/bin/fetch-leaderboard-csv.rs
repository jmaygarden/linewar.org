use leaderboard_scraper::scrape_leaderboard;

#[tokio::main]
async fn main() {
    let leaderboard = scrape_leaderboard()
        .await
        .expect("Error fetching the Line War leaderboard.");

    println!("Rank, Avatar, Name, Rating, Wins, Losses");

    for entry in leaderboard.iter() {
        println!(
            "{}, \"{}\", \"{}\", {}, {}, {}",
            entry.rank, entry.avatar, entry.name, entry.rating, entry.wins, entry.losses,
        )
    }
}
