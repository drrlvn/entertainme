use super::{Error, Result};
use std::fmt::Display;

static BASE_URL: once_cell::sync::Lazy<reqwest::Url> =
    once_cell::sync::Lazy::new(|| reqwest::Url::parse("https://howlongtobeat.com/").unwrap());

#[derive(Debug)]
pub struct GameData {
    pub name: String,
    pub main_story: Option<f64>,
    pub main_plus_extra: Option<f64>,
    pub completionist: Option<f64>,
}

impl GameData {
    pub async fn get(name: String) -> Result<Self> {
        let res = reqwest::Client::new()
            .post(BASE_URL.join("search_results").unwrap())
            .form(&[("queryString", name.as_ref()), ("t", "games")])
            .send()
            .await?
            .error_for_status()?;

        let fragment = scraper::Html::parse_fragment(&res.text().await?);
        let li_selector = scraper::Selector::parse("li").unwrap();
        let h3_selector = scraper::Selector::parse("h3 a").unwrap();
        let time_selector = scraper::Selector::parse("div.search_list_tidbit").unwrap();
        for li in fragment.select(&li_selector) {
            let e3 = li.select(&h3_selector).next();
            if e3.is_none() {
                continue;
            }
            let e3 = e3.unwrap();
            let result_name = e3.inner_html();
            if result_name.to_lowercase() != name {
                continue;
            }

            let mut divs = li.select(&time_selector);
            divs.next();
            let main_story = parse_time(divs.next().unwrap().inner_html());
            divs.next();
            let main_plus_extra = parse_time(divs.next().unwrap().inner_html());
            divs.next();
            let completionist = parse_time(divs.next().unwrap().inner_html());

            return Ok(GameData {
                name: result_name,
                main_story,
                main_plus_extra,
                completionist,
            });
        }

        Err(Error::NotFound)
    }
}

fn parse_time(time: String) -> Option<f64> {
    time.replace("½", ".5")
        .split(' ')
        .next()
        .map(|s| s.parse::<f64>().ok())
        .flatten()
}

impl std::fmt::Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        format_time(f, "Main Story", &self.main_story, &mut first)?;
        format_time(f, "Main + Extra", &self.main_plus_extra, &mut first)?;
        format_time(f, "Completionist", &self.completionist, &mut first)?;

        if first {
            write!(f, "None found")?;
        }

        Ok(())
    }
}

fn format_time(
    f: &mut std::fmt::Formatter<'_>,
    name: impl AsRef<str>,
    opt: &Option<impl Display>,
    first: &mut bool,
) -> std::fmt::Result {
    if let Some(time) = opt {
        write!(
            f,
            "{}{} - {} hours",
            if !*first { ", " } else { "" },
            name.as_ref(),
            time
        )?;
        *first = false;
    }
    Ok(())
}
