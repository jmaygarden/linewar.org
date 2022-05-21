use crate::{Error, Result};

const LEADERBOARD_INDEX_URL: &str = "https://linewar.com/Leaderboard/Index";
const USER_AGENT: &str = "linewar.org";

pub async fn fetch_leaderboard(page: impl Into<Option<u32>>) -> Result<String> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let response = client
        .get(LEADERBOARD_INDEX_URL)
        .query(&[("page", page.into().unwrap_or(1))])
        .send()
        .await
        .map_err(Error::from)?;

    response.text().await.map_err(Error::from)
}

#[cfg(test)]
mod test {
    use super::fetch_leaderboard;

    #[tokio::test]
    async fn test_fetch_index() {
        let _ = fetch_leaderboard(None).await.unwrap();
    }
}
