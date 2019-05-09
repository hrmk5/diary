use std::fs;
use std::process::Command;
use std::path::Path;

use clap;
use chrono::{Local};
use ansi_term::Colour::*;
use regex::Regex;

use config::Config;
use utils::*;

pub fn list(directory: &str, config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let head_id = get_head_id(directory)?;
    let page_count = match matches.value_of("n") {
        Some(n) => n.parse::<u32>().unwrap_or(config.list_max_count),
        None => config.list_max_count,
    };
    let skip = match matches.value_of("skip") {
        Some(skip) => skip.parse::<i32>().unwrap_or(0),
        None => 0,
    };

    let mut prev_id = head_id;
    let mut i = 0;
    loop {
        if i >= page_count as i32 + skip {
            break;
        }

        if prev_id == "NULL" {
            break;
        }

        let page = get_page_by_id(directory, &prev_id)?;

        if i >= skip {
            println!("{} ({})", page.header.title, Yellow.paint(page.id));
        }

        prev_id = page.header.prev;
        i += 1;
    }

    Ok(())
}

pub fn create_new(directory: &str, config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let (id, memo) = match matches.value_of("id") {
        Some(id) => (id.to_string(), true),
        None => {
            // Return current date
            let now = Local::now();
            (now.format("%Y-%m-%d").to_string(), false)
        },
    };

    let page = TemporaryPage {
        header: TemporaryPageHeader {
            title: id.clone(),
            insert_title: true,
            memo,
        },
        text: String::new(),
    };

    create_new_page(directory, &id, &config.editor, &page)?;

    Ok(())
}


pub fn edit(directory: &str, config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let id = match matches.value_of("id") {
        Some(id) => id.to_string(),
        None => {
            // Return current date
            let now = Local::now();
            now.format("%Y-%m-%d").to_string()
        },
    };

    edit_page_by_id(directory, &id, &config.editor)?;

    Ok(())
}

pub fn config(directory: &str, config: &Config, _matches: &clap::ArgMatches) -> Result<(), String> {
    let config_path = Path::new(directory).join("config.toml");

    // Execute editor
    let mut command =
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/c", &format!("{} {}", &config.editor, config_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `cmd /c {}`: {}", &config.editor, err))?
        } else {
            Command::new("sh")
                .args(&["-c", &format!("{} {}", &config.editor, config_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `sh -c {}`: {}", &config.editor, err))?
        };

    let status = command.wait()
        .map_err(|err| format!("Unable to wait editor `{}`: {}", &config.editor, err))?;

    // Error if exit code is not 0
    if !status.success() {
        return Err(format!("Failed editor `{}`", config.editor));
    }

    Ok(())
}

pub fn diary(directory: &str, config: &Config, _matches: &clap::ArgMatches) -> Result<(), String> {
    let now = Local::now();
    let id = now.format("%Y-%m-%d").to_string();

    let today_page_path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    if today_page_path.exists() {
        // Edit if today page file exists
        edit_page_by_id(directory, &id, &config.editor)?;
    } else {
        let page = TemporaryPage {
            header: TemporaryPageHeader {
                title: id.clone(),
                insert_title: true,
                memo: false,
            },
            text: String::new(),
        };
        // Create new if today page file does not exists
        create_new_page(directory, &id, &config.editor, &page)?;
    }

    Ok(())
}

pub fn show(directory: &str, _config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let mut id = match matches.value_of("id") {
        Some(id) => id.to_string(),
        None => {
            // Return current date
            let now = Local::now();
            now.format("%Y-%m-%d").to_string()
        },
    };

    let path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    if !path.exists() {
        // search by regex
        let re = Regex::new(&id).map_err(|err| format!("Invalid regex: {}", err))?;
        let pages_dir = Path::new(directory).join(PAGES_DIR);
        let path_list = fs::read_dir(&pages_dir)
            .map_err(|err| format!("Unable to list files in directory `{}`: {}", pages_dir.to_string_lossy(), err))?;

        let mut matched = false;
        for path in path_list {
            let path = path.unwrap().path();
            let name = path.file_stem().unwrap().to_str().unwrap();
            if re.is_match(name) {
                id = name.to_string();
                matched = true;
            }
        }

        if !matched {
            return Err(String::from("Not found"));
        }
    }

    let page = get_page_by_id(directory, &id)?;

    if page.header.insert_title {
        println!("# {}\n", page.header.title);
    }

    println!("{}", page.text);

    Ok(())
}

pub fn search(directory: &str, _config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let query = matches.value_of("query").unwrap();
    let head_id = get_head_id(directory)?;

    let mut prev_id = head_id;
    loop {
        if prev_id == "NULL" {
            break;
        }

        let page = get_page_by_id(directory, &prev_id)?;

        // Search
        if page.text.contains(query) {
            println!("{} ({})", page.header.title, Yellow.paint(page.id));
        }

        prev_id = page.header.prev;
    }

    Ok(())
}

pub fn editid(directory: &str, _config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let prev_id = matches.value_of("prev_id").unwrap();
    let next_id = matches.value_of("next_id").unwrap();

    edit_id(directory, prev_id, next_id)?;

    Ok(())
}
