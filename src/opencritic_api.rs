use super::{Error, Result};
use serde::Deserialize;

const BASE_URL: once_cell::sync::Lazy<reqwest::Url> =
    once_cell::sync::Lazy::new(|| reqwest::Url::parse("https://api.opencritic.com/api/").unwrap());

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: u64,
    name: String,
    dist: f64,
}

#[derive(Debug, Deserialize)]
struct GameResponse {
    tier: String,
    percentile: u8,
    #[serde(rename = "percentRecommended")]
    percent_recommended: f64,
    #[serde(rename = "topCriticScore")]
    top_critic_score: f64,
}

#[derive(Debug)]
pub struct GameData {
    pub name: String,
    pub tier: String,
    pub percentile: u8,
    pub percent_recommended: f64,
    pub top_critic_score: f64,
}

impl GameData {
    pub async fn get(name: impl AsRef<str>) -> Result<Self> {
        let search_results: Vec<SearchResult> = reqwest::Client::new()
            .get(BASE_URL.join("game/search").unwrap())
            .query(&[("criteria", name.as_ref())])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        for search_result in search_results {
            if search_result.dist != 0. {
                continue;
            }

            let game_response: GameResponse = reqwest::get(
                BASE_URL
                    .join("game/")
                    .unwrap()
                    .join(&search_result.id.to_string())
                    .unwrap(),
            )
            .await?
            .error_for_status()?
            .json()
            .await?;

            return Ok(Self {
                name: search_result.name,
                tier: game_response.tier,
                percentile: game_response.percentile,
                percent_recommended: game_response.percent_recommended,
                top_critic_score: game_response.top_critic_score,
            });
        }

        Err(Error::NotFound)
    }
}

impl std::fmt::Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (top {}%, {:.02} top critic average, {:.02}% critics recommend)",
            self.tier,
            100 - self.percentile,
            self.top_critic_score,
            self.percent_recommended
        )
    }
}
