use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;

mod games;

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

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(required(true))]
    names: Vec<Names>,
}

#[derive(Debug)]
pub struct Names(Vec<String>);

impl FromStr for Names {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Names(s.split('|').map(str::to_string).collect()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let games_data = games::get_data(&opt.names).await?;
    for game_data in games_data {
        println!("{}", game_data);
    }
    Ok(())
}
