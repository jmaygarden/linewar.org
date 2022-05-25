use crate::{fetch::USER_AGENT, Error, Result};
use tracing::info;

pub struct Steam {
    client: reqwest::Client,
    key: String,
    session_id: Option<String>,
}

impl Steam {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .build()?;
        let key = std::env::var("STEAM_API_KEY").map_err(|_| Error::SteamApiKeyNotSet)?;
        let session_id = None;

        Ok(Steam {
            client,
            key,
            session_id,
        })
    }

    pub async fn start_session(&mut self) -> Result<()> {
        const URL: &str = "https://steamcommunity.com/search/users";
        let response = self.client.get(URL).send().await.map_err(Error::from)?;
        let cookie = response
            .cookies()
            .find(|cookie| cookie.name() == "sessionid")
            .ok_or_else(|| Error::SessionIdNotFound)?;
        let session_id = cookie.value().to_string();

        info!("sessionid = '{session_id}'");
        self.session_id.replace(session_id);

        Ok(())
    }

    pub async fn search_users(&self, search_text: &str) -> Result<String> {
        const URL: &str = "https://steamcommunity.com/search/SearchCommunityAjax";
        let session_id = self
            .session_id
            .as_ref()
            .ok_or_else(|| Error::SessionIdNotFound)?;
        let response = self
            .client
            .get(URL)
            .query(&[
                ("text", search_text),
                ("filter", "users"),
                ("page", "1"),
                ("sessionid", session_id.as_str()),
            ])
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()?;

        response
            .json::<UserSearchResponse>()
            .await
            .map(|response| response.html)
            .map_err(Error::from)
    }

    pub async fn resolve_id(&self, vanityurl: &str) -> Result<u64> {
        const URL: &str = "http://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001";
        let response = self
            .client
            .get(URL)
            .query(&[("key", self.key.as_str()), ("vanityurl", vanityurl)])
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()?;
        let response = response
            .json::<ResolvedIdResponse>()
            .await
            .map_err(Error::from)?
            .response;

        if response.success != 0 {
            response.steamid.parse().map_err(Error::from)
        } else {
            Err(Error::UserNotFound)
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolvedId {
    steamid: String,
    success: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolvedIdResponse {
    response: ResolvedId,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserSearchResponse {
    pub success: i32,
    pub search_text: String,
    pub search_result_count: i32,
    pub search_filter: String,
    pub search_page: i32,
    pub html: String,
}

#[cfg(test)]
mod test {
    use super::Steam;

    #[tokio::test]
    async fn test_steam_search_users() {
        let mut steam = Steam::new().expect("error initializing Steam client");

        steam
            .start_session()
            .await
            .expect("failed to fetch the session ID cookie");
        steam.search_users("monjardin").await.unwrap();
    }

    #[tokio::test]
    async fn test_steam_resolve_id() {
        dotenv::dotenv().unwrap();
        let steam = Steam::new().expect("error initializing Steam client");
        let steam_id = steam.resolve_id("monjardin1").await.unwrap();
        assert_eq!(steam_id, 76561198020520825u64);
    }
}
