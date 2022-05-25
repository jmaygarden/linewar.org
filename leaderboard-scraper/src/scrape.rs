use crate::{Entry, Error, Result, SteamId, SteamUser};
use scraper::{ElementRef, Html, Selector};
use tracing::error;

fn parse_leaderboard_row(row: ElementRef) -> Result<Entry> {
    let columns = Selector::parse("td").expect("bad selector");
    let img = Selector::parse("img").expect("bad selector");
    let mut iter = row.select(&columns);

    let rank = iter
        .next()
        .ok_or_else(|| Error::ParseError("rank column missing".into()))?
        .inner_html()
        .trim_start_matches('#')
        .parse()
        .map_err(|err| Error::ParseError(format!("{err:?}").into()))?;
    let avatar = iter
        .next()
        .ok_or_else(|| Error::ParseError("avatar column missing".into()))?
        .select(&img)
        .next()
        .ok_or_else(|| Error::ParseError("avatar image missing".into()))?
        .value()
        .attr("src")
        .unwrap_or_default()
        .into();
    let name = iter
        .next()
        .ok_or_else(|| Error::ParseError("rank column missing".into()))?
        .inner_html()
        .trim()
        .into();
    let rating = iter
        .next()
        .ok_or_else(|| Error::ParseError("rating column missing".into()))?
        .inner_html()
        .parse()
        .map_err(|err| Error::ParseError(format!("{err:?}").into()))?;
    let wins = iter
        .next()
        .ok_or_else(|| Error::ParseError("wins column missing".into()))?
        .inner_html()
        .parse()
        .map_err(|err| Error::ParseError(format!("{err:?}").into()))?;
    let losses = iter
        .next()
        .ok_or_else(|| Error::ParseError("losses column missing".into()))?
        .inner_html()
        .parse()
        .map_err(|err| Error::ParseError(format!("{err:?}").into()))?;

    Ok(Entry {
        rank,
        avatar,
        name,
        rating,
        wins,
        losses,
    })
}

pub fn scrape_leaderboard(html: &str) -> Result<Vec<Entry>> {
    let document = Html::parse_document(html);
    let table = Selector::parse("table.rankTable").expect("bad selector");
    let rows = Selector::parse("tr").expect("bad selector");
    let table = document
        .select(&table)
        .next()
        .ok_or_else(|| Error::ParseError("rank table not found".into()))?;
    let mut list = Vec::new();

    for row in table.select(&rows) {
        match parse_leaderboard_row(row) {
            Ok(entry) => list.push(entry),
            Err(err) => error!("leaderboard parse error: {err:?}"),
        }
    }

    Ok(list)
}

impl TryFrom<&str> for SteamId {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let error = || Error::ParseError("invalid Steam user URL".into());
        let (prefix, id) = value.rsplit_once('/').ok_or_else(error)?;
        let (_, prefix) = prefix.rsplit_once('/').ok_or_else(error)?;

        match prefix {
            "id" => return Ok(SteamId::Url(id.to_string())),
            "profiles" => return Ok(SteamId::Id(id.parse().map_err(Error::from)?)),
            _ => return Err(error()),
        };
    }
}

fn parse_steam_user_row(row: ElementRef) -> Result<SteamUser> {
    let name_selector = Selector::parse("a.searchPersonaName").expect("bad selector");
    let avatar_selector = Selector::parse("div.avatarMedium > a > img").expect("bad selector");
    let id_selector = Selector::parse("div.avatarMedium > a").expect("bad selector");
    let name = row
        .select(&name_selector)
        .next()
        .ok_or_else(|| Error::ParseError("user name not found".into()))?
        .inner_html();
    let avatar = row
        .select(&avatar_selector)
        .next()
        .ok_or_else(|| Error::ParseError("user avatar not found".into()))?
        .value()
        .attr("src")
        .ok_or_else(|| Error::ParseError("user avatar not found".into()))?
        .to_string();
    let id = row
        .select(&id_selector)
        .next()
        .ok_or_else(|| Error::ParseError("user avatar not found".into()))?
        .value()
        .attr("href")
        .ok_or_else(|| Error::ParseError("user avatar not found".into()))?
        .try_into()?;

    Ok(SteamUser { name, avatar, id })
}

pub fn scrape_steam_users<T, F>(html: &str, mut filter_map: F) -> Result<Vec<T>>
where
    F: FnMut(SteamUser) -> Option<T>,
{
    let document = Html::parse_document(html);
    let rows = Selector::parse("div.search_row").expect("bad selector");

    Ok(document
        .select(&rows)
        .filter_map(|row| match parse_steam_user_row(row) {
            Ok(user) => filter_map(user),
            Err(err) => {
                error!("Steam search users parse error: {err:?}");
                None
            }
        })
        .collect())
}

#[cfg(test)]
mod test {
    use super::scrape_leaderboard;
    use crate::{scrape::scrape_steam_users, SteamId};

    #[test]
    fn test_scrape_index() {
        const HTML: &str = r#"
        <html>
            <body>
                <table class="rankTable">
                    <tr>
                        <td>#1</td>
                        <td class="steam-avatar-col"><img class="steam-avatar" src="https://steamcdn-a.akamaihd.net/steamcommunity/public/images/avatars/90/90588b809dab3942f510d8d438779fbcb20a7b29_medium.jpg" /></td>
                        <td>Orbnet</td>
                        <td title="43.9591045045235">43.96</td>
                        <td>73</td>
                        <td>9</td>
                        <td>
                        </td>
                    </tr>
                </table>
            </body>
        </html>
        "#;

        let entries = scrape_leaderboard(HTML).unwrap();
        assert_eq!(entries.len(), 1);

        let entry = &entries[0];
        assert_eq!(entry.rank, 1);
        assert_eq!(
            entry.avatar.as_str(),
            "https://steamcdn-a.akamaihd.net/steamcommunity/public/images/avatars/90/90588b809dab3942f510d8d438779fbcb20a7b29_medium.jpg"
        );
        assert_eq!(entry.name.as_str(), "Orbnet");
        assert_eq!(entry.rating, 43.96);
        assert_eq!(entry.wins, 73);
        assert_eq!(entry.losses, 9);
    }

    #[test]
    fn test_scrape_steam_users() {
        const HTML: &str = r##"
<div class="community_search_results_container">
  <div class="maincontent">
    <div id="search_results" style="opacity: 1;">
      <div class="search_row" data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}">
        <div class="mediumHolder_default" data-miniprofile="60255097" style="float:left;">
            <div class="avatarMedium"><a href="https://steamcommunity.com/id/monjardin1"><img src="https://avatars.akamai.steamstatic.com/c4c2152dfa696da706cd5484dc0d4de10fa062a0_medium.jpg"></a></div>
        </div>
        <div class="searchPersonaInfo">
            <a class="searchPersonaName" href="https://steamcommunity.com/id/monjardin1">monjardin</a><br>
            &nbsp;			
        </div>
        <div class="search_result_friend">
        </div>
        <div style="clear:right"></div>
        <div style="clear:both"></div>
        <div class="search_match_info">
            <div>Custom URL: steamcommunity.com/id/<span style="color: whitesmoke">monjardin1</span></div>
        </div>
      </div>
      <div class="search_row" data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}">
        <div class="mediumHolder_default" data-miniprofile="429753293" style="float:left;">
            <div class="avatarMedium"><a href="https://steamcommunity.com/profiles/76561198390019021"><img src="https://avatars.akamai.steamstatic.com/f5c43cf3801a81c5f9f7e2a791a5f6b0b705bcc1_medium.jpg"></a></div>
        </div>
        <div class="searchPersonaInfo">
            <a class="searchPersonaName" href="https://steamcommunity.com/profiles/76561198390019021">kiel.monjardin</a><br>
            Ezekiel Monjardin<br>			Manila, Manila, Philippines&nbsp;<img style="margin-bottom:-2px" src="https://community.akamai.steamstatic.com/public/images/countryflags/ph.gif" border="0">			
        </div>
        <div class="search_result_friend">
        </div>
        <div style="clear:right"></div>
        <div style="clear:both"></div>
      </div>
    </div>
  </div>
</div>
</div>
        "##;

        let users = scrape_steam_users(HTML, Some).unwrap();
        assert_eq!(users.len(), 2);

        let user = &users[0];
        assert_eq!(user.name, "monjardin");
        assert_eq!(user.avatar, "https://avatars.akamai.steamstatic.com/c4c2152dfa696da706cd5484dc0d4de10fa062a0_medium.jpg");
        assert_eq!(user.id, SteamId::Url("monjardin1".into()));

        let user = &users[1];
        assert_eq!(user.name, "kiel.monjardin");
        assert_eq!(user.avatar, "https://avatars.akamai.steamstatic.com/f5c43cf3801a81c5f9f7e2a791a5f6b0b705bcc1_medium.jpg");
        assert_eq!(user.id, SteamId::Id(76561198390019021u64));
    }
}
