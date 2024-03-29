use crate::{
    fetch::USER_AGENT, parse_avatar_url, scrape::scrape_steam_users, Error, Result, SteamId,
    SteamUser,
};
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

    pub async fn search_users(&self, search_text: &str, page: i32) -> Result<UserSearchResponse> {
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
                ("page", page.to_string().as_str()),
                ("sessionid", session_id.as_str()),
            ])
            .send()
            .await
            .map_err(Error::from)?
            .error_for_status()?;

        response
            .json::<UserSearchResponse>()
            .await
            .map_err(Error::from)
    }

    pub async fn find_id_with_avatar(
        &self,
        name: &str,
        avatar_hash: &str,
        depth: i32,
    ) -> Result<SteamId> {
        let filter_map = |user: SteamUser| {
            if user.name != name {
                return None;
            }

            parse_avatar_url(user.avatar.as_str()).and_then(|hash| {
                if hash == avatar_hash {
                    Some(user.id)
                } else {
                    None
                }
            })
        };

        for page in 0..depth {
            let response = self.search_users(name, page).await?;

            if response.search_result_count == 0 {
                return Err(Error::UserNotFound);
            } else if let Ok(user) = scrape_steam_users(response.html.as_str(), filter_map)
                .and_then(|list| list.into_iter().next().ok_or(Error::UserNotFound))
            {
                return Ok(user);
            }
        }

        Err(Error::SearchAborted)
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
    pub search_page: serde_json::Value,
    pub html: String,
}

#[cfg(test)]
mod test {
    use super::Steam;

    #[tokio::test]
    async fn test_steam_search_users() {
        dotenv::dotenv().unwrap();
        let mut steam = Steam::new().expect("error initializing Steam client");

        steam
            .start_session()
            .await
            .expect("failed to fetch the session ID cookie");
        steam.search_users("monjardin", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_steam_resolve_id() {
        dotenv::dotenv().unwrap();
        let steam = Steam::new().expect("error initializing Steam client");
        let steam_id = steam.resolve_id("monjardin1").await.unwrap();
        assert_eq!(steam_id, 76561198020520825u64);
    }
}
