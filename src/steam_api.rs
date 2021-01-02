use super::{Error, Result};
use futures::future::TryFutureExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use tokio::try_join;

const BASE_URL: once_cell::sync::Lazy<reqwest::Url> =
    once_cell::sync::Lazy::new(|| reqwest::Url::parse("https://store.steampowered.com/").unwrap());

#[derive(Debug, Deserialize)]
struct App {
    appid: u64,
    name: String,
}

static APP_MAP: once_cell::sync::Lazy<HashMap<String, Vec<u64>>> = once_cell::sync::Lazy::new(|| {
    let apps: Vec<App> = serde_json::from_reader(BufReader::new(
        File::open(concat!(env!("OUT_DIR"), "/steam_app_map.json")).unwrap(),
    ))
    .unwrap();
    let mut app_map = HashMap::with_capacity(apps.len());
    for app in apps {
        app_map
            .entry(app.name.trim().to_lowercase().to_string())
            .or_insert_with(|| Vec::new())
            .push(app.appid);
    }
    app_map
});

#[derive(Debug, Deserialize)]
struct AppDetails {
    success: bool,
    data: AppData,
}

#[derive(Debug, Deserialize)]
struct AppData {
    #[serde(rename = "type")]
    type_: String,
    name: String,
    metacritic: Option<MetacriticData>,
}

#[derive(Debug, Deserialize)]
struct MetacriticData {
    score: u8,
    // url: String,
}

#[derive(Debug)]
pub struct GameData {
    pub name: String,
    pub review_data: ReviewData,
    pub metacritic_score: Option<u8>,
}

impl GameData {
    pub async fn get(name: impl AsRef<str>) -> Result<Self> {
        let ids = match APP_MAP.get(name.as_ref()) {
            Some(v) => v,
            None => return Err(Error::NotFound),
        };

        for id in ids {
            let (json, review_data) = try_join!(
                async {
                    reqwest::Client::new()
                        .get(BASE_URL.join("api/appdetails").unwrap())
                        .query(&[("appids", id)])
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<serde_json::Value>()
                        .err_into()
                        .await
                },
                ReviewData::get(*id)
            )?;

            let app_details = AppDetails::deserialize(&json[id.to_string()])?;
            if !app_details.success {
                return Err(Error::ApiUnsuccessful);
            }
            let app_data = app_details.data;
            if app_data.type_ != "game" {
                continue;
            }

            return Ok(Self {
                name: app_data.name,
                review_data,
                metacritic_score: app_data.metacritic.and_then(|m| Some(m.score)),
            });
        }

        return Err(Error::NotFound);
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.review_data)?;
        if let Some(metacritic) = self.metacritic_score {
            write!(f, "\nMetacritic: {}%", metacritic)?
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct AppReviews {
    success: u8,
    query_summary: AppReviewSummary,
}

#[derive(Debug, Deserialize)]
struct AppReviewSummary {
    review_score_desc: String,
    total_positive: u64,
    total_negative: u64,
    total_reviews: u64,
}

#[derive(Debug)]
pub struct ReviewData {
    pub description: String,
    pub positive: u64,
    pub negative: u64,
    pub total: u64,
}

impl ReviewData {
    async fn get(id: u64) -> Result<Self> {
        let client = reqwest::Client::new();
        let res = client
            .get(BASE_URL.join("appreviews/").unwrap().join(&id.to_string()).unwrap())
            .query(&[("json", "1"), ("language", "all"), ("num_per_page", "0")])
            .send()
            .await?
            .error_for_status()?;
        let app_reviews = res.json::<AppReviews>().await?;
        if app_reviews.success != 1 {
            return Err(Error::ApiUnsuccessful);
        }
        Ok(Self {
            description: app_reviews.query_summary.review_score_desc,
            positive: app_reviews.query_summary.total_positive,
            negative: app_reviews.query_summary.total_negative,
            total: app_reviews.query_summary.total_reviews,
        })
    }
}

impl Display for ReviewData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.02}% of {})",
            self.description,
            self.positive as f64 / self.total as f64 * 100f64,
            self.total
        )
    }
}
