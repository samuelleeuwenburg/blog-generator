use chrono::prelude::*;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::post::PostMeta;

#[derive(Debug)]
pub enum TemplateError {
    CantReadTemplateFile,
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TemplateError {}

pub struct Template {
    html: String,
}

impl Template {
    pub fn new(html: String) -> Template {
        Template { html }
    }

    pub fn build_page(
        &self,
        config: &Config,
        content: &str,
        title: &str,
        meta: Option<&PostMeta>,
    ) -> String {
        let html = self.html.to_owned();
        let html = html.replace(&config.selector_content, content);

        // @TODO: prettify this mess of a flow
        let html = if let Some(m) = meta {
            let html = if let Some(description) = &m.description {
                html.replace(&config.selector_description, description)
            } else {
                html
            };

            let html = if let Some(timestamp) = &m.timestamp {
                let human_readable_timestamp = DateTime::from_timestamp(*timestamp as i64, 0)
                    .expect("Bad timestamp!")
                    .format("%Y-%m-%d")
                    .to_string();

                html.replace(&config.selector_timestamp, &human_readable_timestamp)
            } else {
                html
            };

            if let Some(main_image) = &m.main_image {
                html.replace(&config.selector_main_image, main_image)
            } else {
                html
            }
        } else {
            html
        };

        html.replace(&config.selector_title, title)
    }
}

impl TryFrom<String> for Template {
    type Error = TemplateError;

    fn try_from(path_string: String) -> Result<Template, TemplateError> {
        let path = Path::new(&path_string);
        let html = fs::read_to_string(path.join("template.html"))
            .map_err(|_| TemplateError::CantReadTemplateFile)?;

        Ok(Template::new(html))
    }
}
