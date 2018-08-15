#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate toml;
extern crate clap;
extern crate chrono;
extern crate serde;

use std::path::Path;
use std::env;

use clap::{Arg, App, SubCommand};

use config::{Config};

mod page;
mod config;
mod commands;

fn get_app_dir() -> Result<String, failure::Error> {
    if cfg!(target_os = "windows") {
        // Windows では %LOCALAPPDATA%\diary
        let val = env::var("LOCALAPPDATA")?;
        let path = Path::new(&val).join("diary");
        Ok(String::from(path.to_str().unwrap()))
    } else {
        // Linux は $HOME/.config/diary
        let val = env::var("HOME")?;
        let path = Path::new(&val).join(".config").join("diary");
        Ok(String::from(path.to_str().unwrap()))
    }
}

fn main() {
    let matches = App::new("Diary")
        .version("1.0")
        .author("masuke5 <s.zerogoichi@gmail.com>")
        .subcommand(SubCommand::with_name("ls")
                    .about("list diary")
                    .arg(Arg::with_name("--no-color")
                         .help("disable color")))
        .get_matches();

    // 設定を読み込む
    let app_dir = get_app_dir().unwrap();
    let config_path = Path::new(&app_dir).join("config.toml");
    let config_path = config_path.as_path();
    let config = Config::load_from_file(config_path);
    if let Err(err) = config {
        println!("Failed to load config '{}': {}", config_path.to_string_lossy(), err);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("ls") {
        let no_color = matches.is_present("no-color");
        commands::list(&app_dir, no_color);
        return;
    }
}
