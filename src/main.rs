#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate toml;
extern crate clap;
extern crate chrono;
extern crate serde;
extern crate ansi_term;
extern crate regex;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::env;

use clap::{Arg, App, SubCommand};

use config::{Config};

mod page;
mod config;
mod commands;
mod utils;

use utils::{PAGES_DIR, HEAD_FILENAME};

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
    #[cfg(windows)]
    let _enabled = ansi_term::enable_ansi_support();

    let matches = App::new("Diary")
        .version("1.0")
        .author("masuke5 <s.zerogoichi@gmail.com>")
        .subcommand(SubCommand::with_name("ls")
                    .about("list diary")
                    .arg(Arg::with_name("no-color")
                         .long("no-color")
                         .help("disable color"))
                    .arg(Arg::with_name("n")
                         .takes_value(true)
                         .short("n")
                         .help("page count"))
                    .arg(Arg::with_name("skip")
                         .takes_value(true)
                         .long("skip")
                         .help("skip")))
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
        .subcommand(SubCommand::with_name("show")
                    .arg(Arg::with_name("id")
                         .index(1))
                    .about("show detail page"))
        .subcommand(SubCommand::with_name("search")
                    .arg(Arg::with_name("query")
                         .index(1)
                         .required(true))
                    .about("search from all pages"))
        .subcommand(SubCommand::with_name("editid")
                    .arg(Arg::with_name("prev_id")
                         .index(1)
                         .required(true))
                    .arg(Arg::with_name("next_id")
                         .index(2)
                         .required(true))
                    .about("edit id"))
        .get_matches();

    // Load config
    let app_dir = get_app_dir().unwrap();

    // create config.toml and `pages` directory and head file if app directory does not exists
    let app_dir_path = Path::new(&app_dir);
    let config_path = app_dir_path.join("config.toml");
    if !app_dir_path.exists() {
        // create app directory
        if let Err(err) = fs::create_dir(app_dir_path) {
            println!("Unable to create directory `{}`: {}", app_dir_path.to_string_lossy(), err);
            return;
        }

        // create config
        let initial_config_toml = "editor = 'vim'\nlist_max_count = 7";
        let mut config_file = match fs::File::create(&config_path) {
            Ok(file) => file,
            Err(err) => {
                println!("Unable to create config `{}`: {}", config_path.to_string_lossy(), err);
                return;
            },
        };

        if let Err(err) = config_file.write_all(initial_config_toml.as_bytes()) {
            println!("Unable to write initial config toml: {}", err);
            return;
        }

        // create pages directory
        let pages_path = app_dir_path.join(PAGES_DIR);
        if let Err(err) = fs::create_dir(&pages_path) {
            println!("Unable to create directory `{}`: {}", pages_path.to_string_lossy(), err);
            return;
        }

        // create config
        let initial_head = "NULL";
        let head_path = app_dir_path.join(HEAD_FILENAME);
        let mut head_file = match fs::File::create(&head_path) {
            Ok(file) => file,
            Err(err) => {
                println!("Unable to create head file `{}`: {}", head_path.to_string_lossy(), err);
                return;
            },
        };

        if let Err(err) = head_file.write_all(initial_head.as_bytes()) {
            println!("Unable to write initial head id: {}", err);
            return;
        }
    }

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
        Some("show") => commands::show,
        Some("search") => commands::search,
        Some("editid") => commands::editid,
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
