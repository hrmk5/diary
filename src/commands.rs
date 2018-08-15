use std::io::Read;
use std::path::Path;
use std::fs;

use page::{Page};

// ページを保存しているディレクトリ名
const PAGES_DIR: &str = "pages";
// ページファイルの拡張子
const PAGE_EXTENSION: &str = "page";
// 先頭のページ ID を保存しているファイル名
const HEAD_FILENAME: &str = "HEAD";

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

pub fn list(directory: &str, _no_color: bool) -> Result<(), String> {
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

pub fn create_new(_editor: &str) -> Result<(), String> {
    // NEW_PAGE ファイルが存在していなかったらテンプレートを元に作成
    // エディタを起動
    // HEAD ファイルを書き換える
    // NEW_PAGE ファイルを削除
    println!("create new");

    Ok(())
}
