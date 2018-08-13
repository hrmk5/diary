use std::path::Path;
use std::io::Read;

use std::fs;
use toml;
use failure;

#[derive(Debug, Deserialize)]
pub struct Config {
    editor: String,
}

impl Config {
    // TOML 形式のファイルを読み込む
    pub fn load_from_file(filepath: &Path) -> Result<Config, failure::Error> {
        let mut file = fs::File::open(filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }
}
