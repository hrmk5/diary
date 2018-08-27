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

    #[fail(display = "Failed to serialize toml: {}", error)]
    SerializeTomlError {
        error: toml::ser::Error,
    },
}

impl From<toml::de::Error> for PageError {
    fn from(error: toml::de::Error) -> Self {
        PageError::DeserializeTomlError { error }
    }
}

impl From<toml::ser::Error> for PageError {
    fn from(error: toml::ser::Error) -> Self {
        PageError::SerializeTomlError { error }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageHeader {
    pub title: String,
    pub insert_title: bool,
    pub created: DateTime<Utc>,
    pub updated: Vec<DateTime<Utc>>,
    pub memo: bool,
    pub prev: String,
    pub next: String,
}

#[derive(Debug)]
pub struct Page {
    pub id: String,
    pub header: PageHeader,
    pub text: String,
}

impl Page {
    pub fn from_str(s: &str, id: &str) -> Result<Page, PageError> {
        // Split with ---
        let tmp: Vec<&str> = s.splitn(3, "---").collect();
        if tmp.len() < 3 {
            return Err(PageError::ParseError(String::from("Header or text does not exists")));
        }

        let toml_str = tmp[1];
        let text = tmp[2];

        // Deserialize header TOML
        let header: PageHeader = toml::from_str(toml_str)?;

        Ok(Page {
            id: id.to_string(),
            header,
            text: text.to_string().trim().to_string(),
        })
    }

    pub fn to_str(&self) -> Result<String, PageError> {
        let header_toml = toml::to_string(&self.header)?;       

        Ok(format!("---\n{}---\n{}", header_toml, self.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn page_to_str() {
        let page = Page {
            id: "example".to_string(),
            header: PageHeader {
                title: "taitoru".to_string(),
                insert_title: true,
                created: Utc.ymd(2018, 8, 15).and_hms(17, 52, 11),
                updated: vec![Utc.ymd(2018, 8, 15).and_hms(17, 52, 44)],
                memo: true,
                prev: "NULL".to_string(),
                next: "NULL".to_string(),
            },
            text: "本文".to_string(),
        };

        let expected = r#"---
title = "taitoru"
insert_title = true
created = "2018-08-15T17:52:11Z"
updated = ["2018-08-15T17:52:44Z"]
memo = true
prev = "NULL"
next = "NULL"
---
本文"#;

        assert_eq!(page.to_str().unwrap(), expected);
    }
}
