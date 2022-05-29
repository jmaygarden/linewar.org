use dotenv::dotenv;
use leaderboard_scraper::{
    parse_avatar_url, scrape::scrape_steam_users, scrape_leaderboard, steam::Steam, Error, Result,
    SteamId,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut steam = Steam::new().expect("error initializing Steam client");

    steam
        .start_session()
        .await
        .expect("failed to fetch the session ID cookie");

    let leaderboard = scrape_leaderboard()
        .await
        .expect("Error fetching the Line War leaderboard.");

    for entry in leaderboard.into_iter() {
        match find_user(&steam, entry.name.as_str(), entry.avatar.as_str()).await {
            Ok(id) => println!("\"{}\", \"{}\", {}", entry.name, entry.avatar, id),
            Err(_error) => println!("\"{}\", \"{}\",", entry.name, entry.avatar),
        }
    }
}

async fn find_user(steam: &Steam, name: &str, avatar: &str) -> Result<u64> {
    let html = steam
        .search_users(name)
        .await
        .expect("error searching for Steam users");
    let avatar_hash = parse_avatar_url(avatar).ok_or(Error::UserNotFound)?;

    let users = scrape_steam_users(html.as_str(), |user| {
        let user_hash = parse_avatar_url(user.avatar.as_str())?;

        if user.name == name && user_hash == avatar_hash {
            Some(user.id)
        } else {
            None
        }
    })?;

    for id in users.into_iter() {
        let id = resolve_id(&steam, id).await?;

        return Ok(id);
    }

    Err(Error::UserNotFound)
}

async fn resolve_id(steam: &Steam, id: SteamId) -> Result<u64> {
    match id {
        SteamId::Id(value) => Ok(value),
        SteamId::Url(value) => steam.resolve_id(value.as_str()).await,
    }
}
