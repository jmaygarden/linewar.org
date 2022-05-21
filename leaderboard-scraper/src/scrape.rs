use crate::{Entry, Error, Result};
use scraper::{ElementRef, Html, Selector};
use tracing::error;

fn parse_row(row: ElementRef) -> Result<Entry> {
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
        .ok_or_else(|| Error::ParseError(format!("rank table not found").into()))?;
    let mut list = Vec::new();

    for row in table.select(&rows) {
        match parse_row(row) {
            Ok(entry) => list.push(entry),
            Err(err) => error!("leaderboard parse error: {err:?}"),
        }
    }

    Ok(list)
}

#[cfg(test)]
mod test {
    use super::scrape_leaderboard;

    #[test]
    fn test_scrape_index() {
        let html = r#"
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

        let entries = scrape_leaderboard(html).unwrap();
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
}
