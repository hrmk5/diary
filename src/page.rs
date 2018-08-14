use toml;
use chrono::prelude::{DateTime, Utc};

#[derive(Debug, Fail)]
pub enum PageError {
    #[fail(display = "Failed to parse page: {}", _0)]
    ParseError(String),

    #[fail(display = "Failed to deserialize toml: {}", error)]
    DeserializeTomlError {
        error: toml::de::Error,
    },
}

impl From<toml::de::Error> for PageError {
    fn from(error: toml::de::Error) -> Self {
        PageError::DeserializeTomlError { error }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageHeader {
    title: String,
    insert_title: bool,
    author: String,
    created: DateTime<Utc>,
    updated: Vec<DateTime<Utc>>,
    memo: bool,
    prev: String,
    next: String,
}

#[derive(Debug)]
pub struct Page {
    id: String,
    header: PageHeader,
    text: String,
}

impl Page {
    pub fn from_str(s: &str, id: &str) -> Result<Page, PageError> {
        // --- で分割
        let tmp: Vec<&str> = s.splitn(3, "---").collect();
        if tmp.len() < 3 {
            return Err(PageError::ParseError(String::from("Header or text does not exists")));
        }

        let toml_str = tmp[1];
        let text = tmp[2];

        // ヘッダの TOML をデシリアライズ
        let header: PageHeader = toml::from_str(toml_str)?;

        Ok(Page {
            id: id.to_string(),
            header,
            text: text.to_string().trim().to_string(),
        })
    }
}
