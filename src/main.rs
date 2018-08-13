#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate toml;
extern crate failure;
extern crate clap;
extern crate chrono;
extern crate serde;

use std::io;
use std::env;
use std::path::Path;

use clap::{Arg, App, SubCommand};

use page::{PageManager};
use config::{Config};

mod page;
mod config;

fn list(no_color: bool) {
    let loader = PageManager::new("C:/Users/Shinsuke/Documents/diary");
    let headers = match loader.load_headers() {
        Ok(headers) => headers,
        Err(err) => {
            println!("エラーが発生しました: {}", err);
            return;
        },
    };

    println!("{:?}", headers);

    if let Err(err) = loader.write_headers(&headers) {
        println!("エラーが発生しました: {}", err);
        return;
    }
}

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

    println!("{:?}", config);
    
    if let Some(matches) = matches.subcommand_matches("ls") {
        let no_color = matches.is_present("no-color");
        list(no_color);
        return;
    }
}
