use chrono::prelude::*;
use comrak::{markdown_to_html, ComrakOptions};
use serde::Deserialize;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub enum PostError {
    InvalidPostMeta,
    InvalidPostFile,
    CantReadFile,
    CantReadPath,
}

impl fmt::Display for PostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PostError {}

#[derive(Debug)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub html: String,
    pub dir: String,
    pub meta: PostMeta,
}

impl Post {
    pub fn new(slug: String, html: String, dir: String, meta: PostMeta) -> Post {
        let title = match &meta.title {
            Some(title) => title.clone(),
            None => slug.clone(),
        };

        Post {
            title,
            html,
            dir,
            slug,
            meta,
        }
    }

    pub fn render_date(&self) -> String {
        let date = self
            .meta
            .timestamp
            .map(|t| {
                DateTime::from_timestamp(t as i64, 0)
                    .expect("Bad timestamp!")
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or("".into());

        format!("{}", date)
    }

    pub fn render_link(&self) -> String {
        format!("<a href=\"/{}\">{}</a>", self.slug, self.title)
    }
}

impl TryFrom<String> for Post {
    type Error = PostError;

    fn try_from(path_string: String) -> Result<Post, PostError> {
        let path = Path::new(&path_string);

        let markdown =
            fs::read_to_string(path.join("content.md")).map_err(|_| PostError::CantReadFile)?;

        let mut options = ComrakOptions::default();

        options.extension.footnotes = true;
        options.render.unsafe_ = true;

        let html = markdown_to_html(&markdown, &options);

        let dir = path.to_str().unwrap().to_owned();
        let slug = path
            .file_name()
            .ok_or(PostError::CantReadPath)?
            .to_str()
            .unwrap()
            .to_owned();

        let meta: PostMeta = File::open(path.join("meta.json"))
            .map_err(|_| PostError::CantReadFile)
            .and_then(|file| file.try_into())
            .unwrap_or_else(|_| Default::default());

        Ok(Post::new(slug, html, dir, meta))
    }
}

#[derive(Deserialize, Debug)]
pub struct PostMeta {
    pub title: Option<String>,
    pub timestamp: Option<usize>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub main_image: Option<String>,
    pub draft: Option<bool>,
}

impl Default for PostMeta {
    fn default() -> Self {
        PostMeta {
            title: None,
            timestamp: None,
            tags: None,
            description: None,
            keywords: None,
            main_image: None,
            draft: None,
        }
    }
}

impl TryFrom<File> for PostMeta {
    type Error = PostError;

    fn try_from(file: File) -> Result<PostMeta, PostError> {
        let mut buf_reader = BufReader::new(file);
        let mut content = String::new();
        buf_reader
            .read_to_string(&mut content)
            .map_err(|_| PostError::CantReadFile)?;

        serde_json::from_str(&content).map_err(|_| PostError::InvalidPostMeta)
    }
}
