use clap::Parser;
use dotenv::dotenv;
use leaderboard_db::LeaderboardDatabase;
use leaderboard_scraper::{Steam, SteamId};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 1)]
    search_depth: i32,
}

#[tokio::main]
async fn main() {
    let (db, steam) = init().await;
    let args = Args::parse();

    let n = db.index_names().expect("error indexing new names");
    println!("Indexed {n} new names.");

    let n = db
        .hash_avatar_urls()
        .expect("error hashing new avatar URLs");
    println!("Hashed {n} new avatar URLs.");

    let new_players = db.get_new_players().expect("error querying new players");
    println!("Found {} unassociated players", new_players.len());

    for player in new_players.into_iter() {
        let (name, avatar_hash, names_id, avatar_hash_id) = player;
        let result = match steam
            .find_id_with_avatar(&name, &avatar_hash, args.search_depth)
            .await
        {
            Ok(steam_id) => match steam_id {
                SteamId::Id(value) => Ok(value),
                SteamId::Url(value) => steam.resolve_id(value.as_str()).await,
            },
            Err(error) => Err(error),
        };

        match result {
            Ok(steam_id) => {
                match db.associate_player(names_id, avatar_hash_id, steam_id.to_le_bytes().into()) {
                    Ok(n) => {
                        println!("Associated new player({n}): {name} / {avatar_hash} / {steam_id}")
                    }
                    Err(error) => eprintln!(
                        "Error associating player: {name} / {avatar_hash} / {steam_id} / {error:?}"
                    ),
                }
            }
            Err(error) => eprintln!("{name} / {avatar_hash} => {error:?}"),
        }
    }

    let n = db
        .associate_leaderboard()
        .expect("error associating leaderboard entries with Steam players");
    println!("Associated {n} leaderboard entries with players.");
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
