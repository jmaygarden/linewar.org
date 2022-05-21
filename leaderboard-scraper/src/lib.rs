use std::borrow::Cow;

pub mod fetch;
pub mod scrape;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTML parse error: {0}")]
    HtmlParseError(String),
    #[error("parse error: {0}")]
    ParseError(Cow<'static, str>),
    #[error("reqwest error: {0}")]
    RequestError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub rank: u32,
    pub avatar: String,
    pub name: String,
    pub rating: f64,
    pub wins: u32,
    pub losses: u32,
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

#[cfg(test)]
pub(crate) mod tests {
    use super::{fetch, scrape};

    #[tokio::test]
    async fn test_fetch_and_scrape() {
        let html = fetch::fetch_leaderboard(None).await.unwrap();
        let entries = scrape::scrape_leaderboard(html.as_str()).unwrap();

        assert_eq!(entries.len(), 250);
    }
}
