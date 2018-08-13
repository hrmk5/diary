#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate failure;
extern crate clap;
extern crate chrono;
extern crate serde;

mod page;

use std::io;
use clap::{Arg, App, SubCommand};
use page::{PageManager};

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

fn main() {
    let matches = App::new("Diary")
        .version("1.0")
        .author("masuke5 <s.zerogoichi@gmail.com>")
        .subcommand(SubCommand::with_name("ls")
                    .about("list diary")
                    .arg(Arg::with_name("--no-color")
                         .help("disable color")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("ls") {
        let no_color = matches.is_present("no-color");
        list(no_color);
        return;
    }
}
