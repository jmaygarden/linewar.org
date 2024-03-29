use clap::Parser;
use dotenv::dotenv;
use leaderboard_scraper::{
    parse_avatar_url, scrape::scrape_steam_users, steam::Steam, Result, SteamId,
};
use tracing::error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    name: String,
    avatar: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut steam = Steam::new().expect("error initializing Steam client");
    let mut page = 0;

        steam
            .start_session()
            .await
            .expect("failed to fetch the session ID cookie");

    loop {
        eprintln!("page {page}");

        page += 1;

        let search_response = steam
            .search_users(args.name.as_str(), page)
            .await
            .expect("error searching for Steam users");

        if search_response.search_result_count == 0 {
            eprintln!("user not found");
            return;
        }

        let users = scrape_steam_users(search_response.html.as_str(), |user| {
            let hash = parse_avatar_url(user.avatar.as_str())?;

            if user.name == args.name && hash == args.avatar {
                Some(user.id)
            } else {
                None
            }
        })
        .expect("error parsing users");

        for id in users.into_iter() {
            match resolve_id(&steam, id).await {
                Ok(value) => {
                    println!("{}, {}, {value}", args.name, args.avatar);
                    return;
                }
                Err(error) => error!("{error:?}"),
            }
        }
    }
}

async fn resolve_id(steam: &Steam, id: SteamId) -> Result<u64> {
    match id {
        SteamId::Id(value) => Ok(value),
        SteamId::Url(value) => steam.resolve_id(value.as_str()).await,
    }
}
