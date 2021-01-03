use super::{Error, Result};
use serde::Deserialize;
use std::fmt::Display;

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
    percentile: i8,
    #[serde(rename = "percentRecommended")]
    percent_recommended: f64,
    #[serde(rename = "topCriticScore")]
    top_critic_score: f64,
    #[serde(rename = "averageScore")]
    average_score: f64,
}

#[derive(Debug)]
pub struct GameData {
    pub name: String,
    pub tier: String,
    pub percentile: Option<i8>,
    pub percent_recommended: Option<f64>,
    pub top_critic_score: Option<f64>,
    pub average_score: Option<f64>,
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
                percentile: parse_response_num(game_response.percentile),
                percent_recommended: parse_response_num(game_response.percent_recommended),
                top_critic_score: parse_response_num(game_response.top_critic_score),
                average_score: parse_response_num(game_response.average_score),
            });
        }

        Err(Error::NotFound)
    }
}

fn parse_response_num<T: Default + PartialOrd<T>>(num: T) -> Option<T> {
    if num >= T::default() {
        Some(num)
    } else {
        None
    }
}

impl std::fmt::Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.tier.is_empty() {
            write!(f, "{} (", self.tier)?;
        }

        let mut first = true;

        if let Some(percentile) = self.percentile {
            write!(f, "top {}%", 100 - percentile)?;
            first = false;
        }

        format_num(f, " top critic average", &self.top_critic_score, &mut first)?;
        format_num(f, "% recommend", &self.percent_recommended, &mut first)?;
        format_num(f, " average score", &self.average_score, &mut first)?;

        if self.tier.is_empty() {
            if first {
                write!(f, "None found")?;
            }
        } else {
            write!(f, ")")?;
        }

        Ok(())
    }
}

fn format_num(
    f: &mut std::fmt::Formatter<'_>,
    desc: impl AsRef<str>,
    opt: &Option<impl Display>,
    first: &mut bool,
) -> std::fmt::Result {
    if let Some(num) = opt {
        write!(f, "{}{:.02}{}", if !*first { ", " } else { "" }, num, desc.as_ref(),)?;
        *first = false;
    }
    Ok(())
}
