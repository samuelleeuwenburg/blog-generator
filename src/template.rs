use crate::config::Config;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

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
        description: Option<&str>,
    ) -> String {
        let html = self.html.to_owned();
        let html = html.replace(&config.selector_content, content);

        let html = if let Some(description) = description {
            html.replace(&config.selector_description, description)
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
