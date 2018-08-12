
use std::fs;
use std::io;
use std::io::Read;
use std::path::{Path};

use failure;
use serde_json;
use serde_json::{Value};
use chrono::prelude::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct PageHeader {
    title: String,
    insert_title: bool,
    author: String,
    created: DateTime<Utc>,
    updated: Vec<DateTime<Utc>>,
    filename: String,
}

pub struct Page {
    header: PageHeader,
    text: String,
}

pub struct PageLoader {
    directory: String,
}

impl PageLoader {
    pub fn new(directory: &str) -> PageLoader {
        PageLoader{
            directory: String::from(directory),
        }
    }

    // すべてのページを読み込む
    pub fn load_headers(&self) -> Result<Vec<PageHeader>, failure::Error> {
        // JSON ファイルを読み込む
        let mut file = fs::File::open(Path::new(&self.directory).join("headers.json"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // JSON を解析
        let headers: Vec<PageHeader> = serde_json::from_str(&contents)?;

        Ok(headers)
    }
}
