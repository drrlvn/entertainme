use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Response {
    applist: AppList,
}

#[derive(Debug, Deserialize)]
struct AppList {
    apps: Vec<App>,
}

#[derive(Debug, Deserialize, Serialize)]
struct App {
    appid: u64,
    name: String,
}

fn write_app_map(out_path: &Path) {
    let res = reqwest::blocking::get("https://api.steampowered.com/ISteamApps/GetAppList/v2/")
        .expect("Couldn't fetch Steam app list")
        .error_for_status()
        .expect("Couldn't fetch Steam app list");
    let apps = res
        .json::<Response>()
        .expect("Couldn't deserialize Steam app list")
        .applist
        .apps;

    let mut file = BufWriter::new(File::create(out_path.join("steam_app_map.json")).unwrap());
    serde_json::to_writer(&mut file, &apps).unwrap();
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    write_app_map(&out_path);
}
