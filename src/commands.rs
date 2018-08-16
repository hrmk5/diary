use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

use toml;
use clap;
use chrono::{Utc, Local};

use page::{Page, PageHeader, PageError};
use config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct TemporaryPageHeader {
    title: String,
    insert_title: bool,
    author: String,
    memo: bool,
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
struct TemporaryPage {
    header: TemporaryPageHeader,
    text: String,
}

impl TemporaryPage {
    pub fn from_str(s: &str) -> Result<TemporaryPage, PageError> {
        // --- で分割
        let tmp: Vec<&str> = s.splitn(3, "---").collect();
        if tmp.len() < 3 {
            return Err(PageError::ParseError(String::from("Header or text does not exists")));
        }

        let toml_str = tmp[1];
        let text = tmp[2];

        // ヘッダの TOML をデシリアライズ
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

// ページを保存しているディレクトリ名
const PAGES_DIR: &str = "pages";
// ページファイルの拡張子
const PAGE_EXTENSION: &str = "page";
// 先頭のページ ID を保存しているファイル名
const HEAD_FILENAME: &str = "HEAD";
// 一時ファイル
const TEMPORARY_FILE_TO_EDIT: &str = "EDIT_PAGE";

// 使用可能な ID かどうか確認する
fn is_valid_id(id: &str) -> Result<(), String> {
    if id == "" {
        return Err(String::from("empty id is unavaialble"));
    }

    // NULL は使えない
    if id == "NULL" {
        return Err(String::from("`NULL` is unavailable"));
    }

    Ok(())
}

fn read_file(path: &PathBuf) -> Result<String, io::Error> {
    let path = path.as_path();
    let mut file = fs::File::create(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn get_head_id(directory: &str) -> Result<String, String> {
    // 先頭のページ ID を保存しているファイルパスを取得
    let head_filepath = Path::new(directory).join(HEAD_FILENAME);
    let head_filepath = head_filepath.as_path();

    // 先頭のページ ID を取得
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

fn get_page_by_id(directory: &str, id: &str) -> Result<Page, String> {
    // ページファイルのパスを取得
    let filepath = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    let filepath = filepath.as_path();

    // ページファイルを読み込み
    let mut file = match fs::File::open(filepath) {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open page file `{}`: {}", filepath.to_string_lossy(), err)),
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        return Err(format!("Unable to read page file `{}`: {}", filepath.to_string_lossy(), err));
    }

    // ページファイルの文字列をパース
    let page = match Page::from_str(&contents, id) {
        Ok(page) => page,
        Err(err) => return Err(format!("{}", err)),
    };

    Ok(page)
}

fn write_page(directory: &str, id: &str, page: &Page) -> Result<(), String> {
    let path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    let mut page_file = fs::File::create(&path)
        .map_err(|err| format!("Unable to open page file `{}`: {}", path.to_string_lossy(), err))?;

    let page_str = page.to_str().map_err(|err| format!("Unable to serialize page `{}`: {}", id, err))?;

    page_file.write_all(page_str.as_bytes())
        .map_err(|err| format!("Unable to write page to file `{}`: {}", path.to_string_lossy(), err))?;

    Ok(())
}

pub fn list(directory: &str, _no_color: bool) -> Result<(), String> {
    let head_id = get_head_id(directory)?;

    let mut prev_id = head_id;
    loop {
        let page = get_page_by_id(directory, &prev_id)?;

        // ページの情報を出力
        println!("{}: {}", page.id, page.header.title);

        // 前のページ ID が "NULL" だったらループを抜ける
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
            // 現在の日付を取得
            let now = Local::now();
            now.format("%Y-%m-%d").to_string()
        },
    };

    if let Err(err) = is_valid_id(&id) {
        return Err(format!("Invalid ID: {}", err));
    }

    // すでに存在する場合はエラー
    let new_file_path = Path::new(directory).join(PAGES_DIR).join(format!("{}.{}", id, PAGE_EXTENSION));
    if new_file_path.exists() {
        return Err(format!("`{}` already exists", id));
    }

    // 先頭ページの ID を取得
    let head_id = get_head_id(directory)?;
    
    // 変更するページ
    let mut page = Page {
        id: id.clone(),
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

    // 一時ファイルが存在していなかったら作成
    let file_to_edit_path = Path::new(directory).join(TEMPORARY_FILE_TO_EDIT);
    if !file_to_edit_path.exists() {
        let mut file_to_edit = fs::File::create(&file_to_edit_path)
            .map_err(|err| format!("Unable to open temporary file to edit `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;

        let temp_page = TemporaryPage::from_page(&page);

        let page_str = temp_page.to_str().map_err(|err| format!("Unable to initialize page: {}", err))?;
        file_to_edit.write_all(page_str.as_bytes())
            .map_err(|err| format!("Unable to write initial page `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;
    }

    // エディタを起動
    let mut command =
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/c", &format!("{} {}", &config.editor, file_to_edit_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `cmd /c {}`: {}", &config.editor, err))?
        } else {
            Command::new("sh")
                .args(&["-c", &format!("{} {}", &config.editor, file_to_edit_path.to_string_lossy())])
                .spawn()
                .map_err(|err| format!("Unable to execute editor `sh -c {}`: {}", &config.editor, err))?
        };

    let status = command.wait()
        .map_err(|err| format!("Unable to wait editor `{}`: {}", &config.editor, err))?;

    // コマンドが失敗したらエラー
    if !status.success() {
        return Err(format!("Failed editor `{}`", &config.editor));
    }

    // ページファイルに書き込み
    let mut file_to_edit = fs::File::open(&file_to_edit_path)
        .map_err(|err| format!("Unable to open temporary file to edit `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;

    let mut contents = String::new();
    file_to_edit.read_to_string(&mut contents)
        .map_err(|err| format!("Unable to read file to edit `{}`: {}", file_to_edit_path.to_string_lossy(), err))?;

    let temp_page = TemporaryPage::from_str(&contents).map_err(|err| format!("{}", err))?;
    temp_page.apply(&mut page);

    // 更新日時を更新
    page.header.updated.push(Utc::now());

    write_page(directory, &id, &page)?;

    // 先頭ページの next を書き換えて保存する
    let mut head_page = get_page_by_id(directory, &head_id)?;
    head_page.header.next = id.clone();

    write_page(directory, &head_id, &head_page)?;

    // HEAD ファイルを書き換える
    let head_path = Path::new(directory).join(HEAD_FILENAME);   
    let mut head_file = fs::File::create(&head_path)
        .map_err(|err| format!("Unable to open head file `{}`: {}", head_path.to_string_lossy(), err))?;

    head_file.write_all(id.as_bytes())
        .map_err(|err| format!("Unable to write head to file `{}`: {}", head_path.to_string_lossy(), err))?;

    Ok(())
}
