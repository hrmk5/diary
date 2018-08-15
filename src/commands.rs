use std::io::Read;
use std::path::Path;
use std::fs;

use page::{Page};

// ページファイルの拡張子
const PAGE_EXTENSION: &str = "page";

pub fn list(directory: &str, _no_color: bool) {
    // 先頭のページ ID を取得
    let head_filepath = Path::new(directory).join("HEAD");
    let head_filepath = head_filepath.as_path();

    let mut file = match fs::File::open(head_filepath) {
        Ok(file) => file,
        Err(err) => {
            println!("Unable to open HEAD file `{}`: {}", head_filepath.to_string_lossy(), err);
            return;
        },
    };

    let mut head_id = String::new();
    if let Err(err) = file.read_to_string(&mut head_id) {
        println!("Unable to read HEAD file `{}`: {}", head_filepath.to_string_lossy(), err);
        return;
    }

    let mut prev_id = head_id;
    loop {
        // ページファイルを読み込む
        let diary_filepath = Path::new(directory).join("pages").join(format!("{}.{}", prev_id, PAGE_EXTENSION));
        let diary_filepath = diary_filepath.as_path();

        let mut file = match fs::File::open(diary_filepath) {
            Ok(file) => file,
            Err(err) => {
                println!("Unable to open page file `{}`: {}", diary_filepath.to_string_lossy(), err);
                return;
            },
        };

        let mut contents = String::new();
        if let Err(err) = file.read_to_string(&mut contents) {
            println!("Unable to read page file `{}`: {}", diary_filepath.to_string_lossy(), err);
            return;
        }

        // ページファイルの文字列をパース
        let page = match Page::from_str(&contents, &prev_id) {
            Ok(page) => page,
            Err(err) => {
                println!("{}", err);
                return;
            },
        };

        // ページの情報を出力
        println!("{}: {}", page.id, page.header.title);

        // 前のページ ID が "NULL" だったらループを抜ける
        prev_id = page.header.prev;
        if prev_id == "NULL" {
            break;
        }
    }
}
