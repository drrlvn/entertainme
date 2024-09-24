use super::{Error, Result};
use futures::future::{join, join_all, try_join_all};
use std::fmt::Display;

mod opencritic_api;
mod steam_api;

#[derive(Debug)]
pub struct GameData {
    pub steam: Option<steam_api::GameData>,
    pub opencritic: Option<opencritic_api::GameData>,
}

impl GameData {
    async fn get(names: &super::Names) -> Result<Self> {
        let results = join_all(names.0.iter().map(|name| {
            join(
                steam_api::GameData::get(name.clone()),
                opencritic_api::GameData::get(name.clone()),
            )
        }))
        .await;

        let mut game_data = GameData {
            steam: None,
            opencritic: None,
        };

        for (steam, opencritic) in results {
            let (steam, opencritic) = (parse_result(steam)?, parse_result(opencritic)?);

            game_data.steam = game_data.steam.or(steam);
            game_data.opencritic = game_data.opencritic.or(opencritic);

            if game_data.steam.is_some() && game_data.opencritic.is_some() {
                break;
            }
        }

        if game_data.steam.is_some() || game_data.opencritic.is_some() {
            Ok(game_data)
        } else {
            Err(Error::NotFound)
        }
    }

    fn name(&self) -> &str {
        if let Some(steam) = &self.steam {
            &steam.name
        } else if let Some(opencritic) = &self.opencritic {
            &opencritic.name
        } else {
            ""
        }
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name())?;
        format_option(f, "Steam", &self.steam)?;
        format_option(f, "Opencritic", &self.opencritic)
    }
}

fn format_option(
    f: &mut std::fmt::Formatter<'_>,
    name: impl AsRef<str>,
    opt: &Option<impl Display>,
) -> std::fmt::Result {
    write!(f, "{}: ", name.as_ref())?;
    if let Some(v) = &opt {
        writeln!(f, "{}", v)
    } else {
        writeln!(f, "Not found")
    }
}

fn parse_result<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(Error::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn get_data(games_names: &[super::Names]) -> Result<Vec<GameData>> {
    try_join_all(games_names.iter().map(GameData::get)).await
}
