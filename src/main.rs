use futures::future::try_join_all;
use std::fmt::Display;
use thiserror::Error;
use tokio::join;

mod howlongtobeat_api;
mod opencritic_api;
mod steam_api;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Deserialization error")]
    Deserialization(#[from] serde_json::Error),
    #[error("Reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("API unsuccesful")]
    ApiUnsuccessful,
    #[error("Not found")]
    NotFound,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct GameData {
    name: String,
    steam: Option<steam_api::GameData>,
    opencritic: Option<opencritic_api::GameData>,
    howlongtobeat: Option<howlongtobeat_api::GameData>,
}

impl GameData {
    async fn get(name: impl AsRef<str>) -> Result<Self> {
        let (steam, opencritic, howlongtobeat) = join!(
            steam_api::GameData::get(&name),
            opencritic_api::GameData::get(&name),
            howlongtobeat_api::GameData::get(&name)
        );
        let (steam, opencritic, howlongtobeat) = (
            parse_result(steam)?,
            parse_result(opencritic)?,
            parse_result(howlongtobeat)?,
        );
        Ok(GameData {
            name: if let Some(steam) = &steam {
                steam.name.clone()
            } else if let Some(opencritic) = &opencritic {
                opencritic.name.clone()
            } else if let Some(howlongtobeat) = &howlongtobeat {
                howlongtobeat.name.clone()
            } else {
                name.as_ref().to_string()
            },
            steam,
            opencritic,
            howlongtobeat,
        })
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

async fn get_data<I, T>(games: I) -> Result<Vec<GameData>>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    Ok(try_join_all(
        games
            .into_iter()
            .map(|name| GameData::get(name.as_ref().to_lowercase())),
    )
    .await?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let games_data = get_data(std::env::args().skip(1)).await?;
    for game_data in games_data {
        println!("{}", game_data);
    }
    Ok(())
}
