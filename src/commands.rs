use std::process::Command;
use std::path::Path;

use clap;
use chrono::{Local};
use colored::*;

use config::Config;
use utils::*;

pub fn list(directory: &str, _config: &Config, _matches: &clap::ArgMatches) -> Result<(), String> {
    let head_id = get_head_id(directory)?;

    let mut prev_id = head_id;
    loop {
        let page = get_page_by_id(directory, &prev_id)?;

        println!("{} ({})", page.header.title, page.id.yellow());

        prev_id = page.header.prev;
        if prev_id == "NULL" {
            break;
        }
    }

    Ok(())
}

pub fn create_new(directory: &str, config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let id = match matches.value_of("id") {
        Some(id) => id.to_string(),
        None => {
            // Return current date
            let now = Local::now();
            now.format("%Y-%m-%d").to_string()
        },
    };

    create_new_page(directory, &id, &config.editor)?;

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
        // Create new if today page file does not exists
        create_new_page(directory, &id, &config.editor)?;
    }

    Ok(())
}

pub fn show(directory: &str, _config: &Config, matches: &clap::ArgMatches) -> Result<(), String> {
    let id = match matches.value_of("id") {
        Some(id) => id.to_string(),
        None => {
            // Return current date
            let now = Local::now();
            now.format("%Y-%m-%d").to_string()
        },
    };

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
        let page = get_page_by_id(directory, &prev_id)?;

        // Search
        if page.text.contains(query) {
            println!("{} ({})", page.header.title, page.id.yellow());
        }

        prev_id = page.header.prev;
        if prev_id == "NULL" {
            break;
        }
    }

    Ok(())
}
