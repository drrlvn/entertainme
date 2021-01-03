use super::{Error, Result};
use futures::future::{join3, join_all, try_join_all};
use std::fmt::Display;

mod howlongtobeat_api;
mod opencritic_api;
mod steam_api;

#[derive(Debug)]
pub struct GameData {
    pub name: String,
    pub steam: Option<steam_api::GameData>,
    pub opencritic: Option<opencritic_api::GameData>,
    pub howlongtobeat: Option<howlongtobeat_api::GameData>,
}

impl GameData {
    async fn get(names: &super::Names) -> Result<Self> {
        let results = join_all(names.0.iter().map(|name| {
            join3(
                steam_api::GameData::get(name.clone()),
                opencritic_api::GameData::get(name.clone()),
                howlongtobeat_api::GameData::get(name.clone()),
            )
        }))
        .await;

        let mut game_data = GameData {
            name: String::new(),
            steam: None,
            opencritic: None,
            howlongtobeat: None,
        };

        for (steam, opencritic, howlongtobeat) in results {
            let (steam, opencritic, howlongtobeat) = (
                parse_result(steam)?,
                parse_result(opencritic)?,
                parse_result(howlongtobeat)?,
            );

            game_data.steam = game_data.steam.or(steam);
            game_data.opencritic = game_data.opencritic.or(opencritic);
            game_data.howlongtobeat = game_data.howlongtobeat.or(howlongtobeat);

            if game_data.steam.is_some() && game_data.opencritic.is_some() && game_data.howlongtobeat.is_some() {
                break;
            }
        }

        game_data.name = if let Some(steam) = &game_data.steam {
            steam.name.clone()
        } else if let Some(opencritic) = &game_data.opencritic {
            opencritic.name.clone()
        } else if let Some(howlongtobeat) = &game_data.howlongtobeat {
            howlongtobeat.name.clone()
        } else {
            names.0[0].clone()
        };

        if game_data.steam.is_some() || game_data.opencritic.is_some() || game_data.howlongtobeat.is_some() {
            Ok(game_data)
        } else {
            Err(Error::NotFound)
        }
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        format_option(f, "Steam", &self.steam)?;
        format_option(f, "Opencritic", &self.opencritic)?;
        format_option(f, "How Long To Beat", &self.howlongtobeat)
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

pub async fn get_data(games_names: &Vec<super::Names>) -> Result<Vec<GameData>> {
    Ok(try_join_all(games_names.into_iter().map(|game_names| GameData::get(&game_names))).await?)
}
