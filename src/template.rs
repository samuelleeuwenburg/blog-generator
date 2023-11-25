use crate::config::Config;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum TemplateError {
    CantReadCssFile,
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
    css: String,
}

impl Template {
    pub fn new(html: String, css: String) -> Template {
        Template { html, css }
    }

    pub fn build_page(&self, config: &Config, content: &str, title: &str) -> String {
        let html = self.html.to_owned();
        let html = html.replace(&config.selector_content, content);
        let html = html.replace(&config.selector_title, title);
        html.replace(&config.selector_css, &self.css)
    }
}

impl TryFrom<String> for Template {
    type Error = TemplateError;

    fn try_from(path_string: String) -> Result<Template, TemplateError> {
        let path = Path::new(&path_string);
        let html = fs::read_to_string(path.join("template.html"))
            .map_err(|_| TemplateError::CantReadTemplateFile)?;
        let css = fs::read_to_string(path.join("style.css"))
            .map_err(|_| TemplateError::CantReadCssFile)?;

        Ok(Template::new(html, css))
    }
}
