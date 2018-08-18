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
mod utils;

fn get_app_dir() -> Result<String, failure::Error> {
    if cfg!(target_os = "windows") {
        // %LOCALAPPDATA%\diary
        let val = env::var("LOCALAPPDATA")?;
        let path = Path::new(&val).join("diary");
        Ok(String::from(path.to_str().unwrap()))
    } else {
        // $HOME/.config/diary
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
        .subcommand(SubCommand::with_name("new")
                    .arg(Arg::with_name("id")
                         .index(1))
                    .about("create new page"))
        .subcommand(SubCommand::with_name("edit")
                    .arg(Arg::with_name("id")
                         .index(1))
                    .about("edit page"))
        .subcommand(SubCommand::with_name("config")
                    .about("edit config file"))
        .get_matches();

    // Load config
    let app_dir = get_app_dir().unwrap();
    let config_path = Path::new(&app_dir).join("config.toml");
    let config_path = config_path.as_path();
    let config = match Config::load_from_file(config_path) {
        Ok(config) => config,
        Err(err) => {
            println!("Failed to load config '{}': {}", config_path.to_string_lossy(), err);
            return;
        },
    };

    let name = matches.subcommand_name();
    let func = match name {
        Some("ls") => commands::list,
        Some("new") => commands::create_new,
        Some("edit") => commands::edit,
        Some("config") => commands::config,
        _ => commands::diary,
    };

    if name == None {
        if let Err(message) = func(&app_dir, &config, &matches) {
            println!("{}", message);
        }
        return;
    }

    if let Some(matches) = matches.subcommand_matches(name.unwrap()) {
        if let Err(message) = func(&app_dir, &config, &matches) {
            println!("{}", message);
        }
    }
}
