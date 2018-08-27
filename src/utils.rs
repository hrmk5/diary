use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

use toml;
use chrono::{Utc};
use page::{Page, PageHeader, PageError};

// Directory name to save pages
pub const PAGES_DIR: &str = "pages";
// Page file extension
pub const PAGE_EXTENSION: &str = "page";
// File name to save head page id
pub const HEAD_FILENAME: &str = "HEAD";
// Temporary file to edit page
pub const TEMPORARY_FILE_TO_EDIT: &str = "EDIT_PAGE";
// Invalid characters in file path
pub const INVALID_CHARACTERS: [&str; 11] = ["\\", "/", ":", ",", ";", "*", "?", "\"", "<", ">", "|"];

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporaryPageHeader {
    pub title: String,
    pub insert_title: bool,
    pub author: String,
    pub memo: bool,
}

impl TemporaryPageHeader {
    pub fn from_pageheader(header: &PageHeader) -> TemporaryPageHeader {
        TemporaryPageHeader {
            title: header.title.clone(),
            insert_title: header.insert_title,
            author: header.author.clone(),
            memo: header.memo,
        }
    }
}

#[derive(Debug)]
pub struct TemporaryPage {
    pub header: TemporaryPageHeader,
    pub text: String,
}

impl TemporaryPage {
    pub fn from_str(s: &str) -> Result<TemporaryPage, PageError> {
        let tmp: Vec<&str> = s.splitn(3, "---").collect();
        if tmp.len() < 3 {
            return Err(PageError::ParseError(String::from("Header or text does not exists")));
        }

        let toml_str = tmp[1];
        let text = tmp[2];

        // deserialize header toml
        let header: TemporaryPageHeader = toml::from_str(toml_str)?;

        Ok(TemporaryPage {
            header,
            text: text.to_string().trim().to_string(),
        })
    }

    pub fn to_str(&self) -> Result<String, PageError> {
        let header_toml = toml::to_string(&self.header)?;       

        Ok(format!("---\n{}---\n{}", header_toml, self.text))
    }

    pub fn from_page(page: &Page) -> TemporaryPage {
        TemporaryPage {
            header: TemporaryPageHeader::from_pageheader(&page.header),
            text: page.text.clone(),
        }
    }

    pub fn apply(&self, page: &mut Page) {
        page.header.title = self.header.title.clone();
        page.header.insert_title = self.header.insert_title.clone();
        page.header.author = self.header.author.clone();
        page.header.memo = self.header.memo.clone();
        page.text = self.text.clone();
    }
}

// Check if id is valid
pub fn is_valid_id(id: &str) -> Result<(), String> {
    if id == "" {
        return Err(String::from("empty id is unavaialble"));
    }

    if id == "NULL" {
        return Err(String::from("`NULL` is unavailable"));
    }

    // if id contains invalid characters
    if INVALID_CHARACTERS.iter().any(|c| id.contains(c)) {
        return Err(format!("invalid character ({})", INVALID_CHARACTERS.join(" ")))
    }

    Ok(())
}

pub fn read_file(path: &PathBuf) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn write_file(path: &PathBuf, contents: &str) -> Result<(), io::Error> {
    let mut file = fs::File::create(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn get_head_id(directory: &str) -> Result<String, String> {
    // Get filepath to save head page id
    let head_filepath = Path::new(directory).join(HEAD_FILENAME);
    let head_filepath = head_filepath.as_path();

    // Get head page id
    let mut file = match fs::File::open(head_filepath) {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open HEAD file `{}`: {}", head_filepath.to_string_lossy(), err)),
    };

    let mut head_id = String::new();
    if let Err(err) = file.read_to_string(&mut head_id) {
        return Err(format!("Unable to read HEAD file `{}`: {}", head_filepath.to_string_lossy(), err));
    }

    Ok(head_id)
}

pub fn get_page_by_id(directory: &str, id: &str) -> Result<Page, String> {
    // Get page filepath
    let filepath = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    let filepath = filepath.as_path();

    // Read page file
    let mut file = match fs::File::open(filepath) {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open page file `{}`: {}", filepath.to_string_lossy(), err)),
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        return Err(format!("Unable to read page file `{}`: {}", filepath.to_string_lossy(), err));
    }

    // Parse page file contents
    let page = match Page::from_str(&contents, id) {
        Ok(page) => page,
        Err(err) => return Err(format!("{}", err)),
    };

    Ok(page)
}

pub fn create_new_page(directory: &str, id: &str, editor: &str, initial_page: &TemporaryPage) -> Result<(), String> {
    if let Err(err) = is_valid_id(&id) {
        return Err(format!("Invalid ID: {}", err));
    }

    // If page file already exists
    let new_file_path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    if new_file_path.exists() {
        return Err(format!("`{}` already exists. use `diary edit {}`", id, id));
    }

    // Get head id
    let head_id = get_head_id(directory)?;
    
    let mut page = Page {
        id: id.clone().to_string(),
        header: PageHeader {
            title: id.to_string(),
            insert_title: true,
            author: "__TEMP1__".to_string(),
            created: Utc::now(),
            updated: Vec::new(),
            memo: true,
            prev: head_id.clone(),
            next: "NULL".to_string(),
        },
        text: String::new(),
    };
    initial_page.apply(&mut page);

    // Edit page
    let page = edit_page(directory, page, editor)?;

    // Write page
    write_page(directory, &id, &page)?;

    // Update next of head page
    if head_id != "NULL" {
        let mut head_page = get_page_by_id(directory, &head_id)?;
        head_page.header.next = id.clone().to_string();

        write_page(directory, &head_id, &head_page)?;
    }

    // Update head file
    let head_path = Path::new(directory).join(HEAD_FILENAME);   
    let mut head_file = fs::File::create(&head_path)
        .map_err(|err| format!("Unable to open head file `{}`: {}", head_path.to_string_lossy(), err))?;

    head_file.write_all(id.as_bytes())
        .map_err(|err| format!("Unable to write head to file `{}`: {}", head_path.to_string_lossy(), err))?;

    Ok(())
}

pub fn edit_page(directory: &str, page: Page, editor: &str) -> Result<Page, String> {
    let mut page = page;

    let temp_page = TemporaryPage::from_page(&page);
    let file_to_edit_path = Path::new(directory).join(TEMPORARY_FILE_TO_EDIT);

    // Write to temporary page file
    let temp_page_str = temp_page.to_str().unwrap();
    write_file(&file_to_edit_path, &temp_page_str)
        .map_err(|err| format!("Unable to write to temporary page file `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;

    // Execute editor
    let mut command =
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/c", &format!("{} {}", editor, file_to_edit_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `cmd /c {}`: {}", editor, err))?
        } else {
            Command::new("sh")
                .args(&["-c", &format!("{} {}", editor, file_to_edit_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `sh -c {}`: {}", editor, err))?
        };

    let status = command.wait()
        .map_err(|err| format!("Unable to wait editor `{}`: {}", editor, err))?;

    // Error if exit code is not 0
    if !status.success() {
        return Err(format!("Failed editor `{}`", editor));
    }

    // Read and parse temporary file
    let contents = read_file(&file_to_edit_path)
        .map_err(|err| format!("Unable to read temporary file `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;

    let temp_page = TemporaryPage::from_str(&contents).map_err(|err| format!("{}", err))?;
    temp_page.apply(&mut page);

    // Update updated times of header
    page.header.updated.push(Utc::now());

    Ok(page)
}

pub fn edit_page_by_id(directory: &str, id: &str, editor: &str) -> Result<(), String> {
    let new_file_path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    if !new_file_path.exists() {
        return Err(format!("`{}` does not exists. use `diary new {}`", id, id));
    }

    // Get page to edit
    let page = get_page_by_id(directory, id)?;

    // Edit page
    let page = edit_page(directory, page, editor)?;

    // Write page
    write_page(directory, &id, &page)?;

    Ok(())
}

pub fn write_page(directory: &str, id: &str, page: &Page) -> Result<(), String> {
    let path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    let mut page_file = fs::File::create(&path)
        .map_err(|err| format!("Unable to open page file `{}`: {}", path.to_string_lossy(), err))?;

    let page_str = page.to_str().map_err(|err| format!("Unable to serialize page `{}`: {}", id, err))?;

    page_file.write_all(page_str.as_bytes())
        .map_err(|err| format!("Unable to write page to file `{}`: {}", path.to_string_lossy(), err))?;

    Ok(())
}
