use serde::Deserialize;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
pub enum ConfigError {
    InvalidConfig,
    CantReadFile,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConfigError {}

#[derive(Deserialize)]
pub struct Config {
    pub dest: String,
    pub title: String,
    pub description: String,
    pub selector_content: String,
    pub selector_title: String,
    pub selector_description: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dest: String::from("./public"),
            title: String::from("Blogname"),
            description: String::from("Blogname"),
            selector_content: String::from("CONTENT"),
            selector_title: String::from("TITLE"),
            selector_description: String::from("DESCRIPTION"),
        }
    }
}

impl TryFrom<File> for Config {
    type Error = ConfigError;

    fn try_from(file: File) -> Result<Config, Self::Error> {
        let mut buf_reader = BufReader::new(file);
        let mut content = String::new();
        buf_reader
            .read_to_string(&mut content)
            .map_err(|_| ConfigError::CantReadFile)?;

        serde_json::from_str(&content).map_err(|_| ConfigError::InvalidConfig)
    }
}
