use std::{borrow::Cow, num::ParseIntError};

pub mod fetch;
pub mod scrape;
pub mod steam;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTML parse error: {0}")]
    HtmlParseError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("parse error: {0}")]
    ParseError(Cow<'static, str>),
    #[error("reqwest error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("user search aborted")]
    SearchAborted,
    #[error("session ID not found")]
    SessionIdNotFound,
    #[error("STEAM_API_KEY must be set")]
    SteamApiKeyNotSet,
    #[error("user not found")]
    UserNotFound,
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::ParseError(err.to_string().into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Entry {
    pub rank: i32,
    pub avatar: String,
    pub name: String,
    pub rating: f32,
    pub wins: i32,
    pub losses: i32,
}

pub use steam::Steam;

#[derive(Debug, PartialEq)]
pub enum SteamId {
    Id(u64),
    Url(String),
}

#[derive(Debug)]
pub struct SteamUser {
    pub name: String,
    pub avatar: String,
    pub id: SteamId,
}

pub async fn scrape_leaderboard() -> Result<Vec<Entry>> {
    let mut list = Vec::new();
    let mut index = 1;

    loop {
        let page = fetch::fetch_leaderboard(index).await?;
        let mut entries = scrape::scrape_leaderboard(page.as_str())?;

        if entries.is_empty() {
            break;
        }

        list.append(&mut entries);
        index += 1;
    }

    Ok(list)
}

pub fn parse_avatar_url(url: &str) -> Option<&str> {
    let (_, suffix) = url.rsplit_once('/')?;
    let (prefix, _) = suffix.split_once('_')?;

    Some(prefix)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{fetch, parse_avatar_url, scrape};

    #[tokio::test]
    async fn test_fetch_and_scrape() {
        let html = fetch::fetch_leaderboard(None).await.unwrap();
        let entries = scrape::scrape_leaderboard(html.as_str()).unwrap();

        assert_eq!(entries.len(), 250);
    }

    #[test]
    fn test_parse_avatar_url() {
        const URL: &str = "https://steamcdn-a.akamaihd.net/steamcommunity/public/images/avatars/c4/c4c2152dfa696da706cd5484dc0d4de10fa062a0_medium.jpg";
        const HASH: &str = "c4c2152dfa696da706cd5484dc0d4de10fa062a0";

        let hash = parse_avatar_url(URL);

        assert_eq!(hash, Some(HASH));
    }
}
