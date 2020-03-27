use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::cmp::Ordering;
use regex::Regex;
use comrak::{markdown_to_html, ComrakOptions};
use serde::Deserialize;
use serde_json;
use clap::{Arg, App};

#[derive(Deserialize)]
struct Config {
    dest: String,
    title: String,
    selector_content: String,
    selector_title: String,
    selector_css: String,
}

fn get_config(src_dir: &str) -> Config {
    println!("reading config..");

    let default_config = Config {
        dest: String::from("./dist"),
        title: String::from("Blogname"),
        selector_content: String::from("CONTENT"),
        selector_title: String::from("TITLE"),
        selector_css: String::from("CSS"),
    };

    open_file(&format!("{}/config.json", src_dir))
        .map_err(|_| "no config file found")
        .and_then(|json| serde_json::from_str(&json).map_err(|_| "config is invalid"))
        .unwrap_or_else(|err| {
            println!("{}, using default one..", err);
            default_config
        })
}

struct Template {
    html: String,
    css: String,
}

impl Template {
    fn new(html: &str, css: &str) -> Template {
        Template {
            html: String::from(html),
            css: String::from(css),
        }
    }

    fn build_page(&self, config: &Config, content: &str, title: &str) -> String {
        let html = self.html.to_owned();
        let html = html.replace(&config.selector_content, &content);
        let html = html.replace(&config.selector_title, &title);
        let html = html.replace(&config.selector_css, &self.css);

        html
    }
}

struct Post {
    slug: String,
    title: String,
    html: String,
    dir: String,
    meta: PostMeta,
}

impl Post {
    fn new(slug: &str, html: &str, dir: &str, meta: PostMeta) -> Post {
        let re_title = Regex::new("<h1>(.+)</h1>").unwrap();
        let title = match re_title.captures(html) {
            Some(capture) => {
                String::from(capture.get(1).unwrap().as_str())
            }
            None => String::from(slug),
        };

        Post {
            meta,
            title,
            slug: String::from(slug),
            html: String::from(html),
            dir: String::from(dir),
        }
    }

    fn render_link(&self) -> String {
        format!("<li><a href=\"/{}\">{}</a></li>\n", self.slug, self.title)
    }
}

fn open_file(path: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;

    Ok(content)
}

#[derive(Deserialize, Debug)]
struct PostMeta {
    timestamp: Option<usize>
}

fn get_meta_from_json_with_default(json: &str) -> PostMeta {
    match serde_json::from_str(json) {
        Ok(post_meta) => post_meta,
        Err(_) => {
            PostMeta { timestamp: None }
        }
    }
}

fn get_posts(src_dir: &str) -> std::io::Result<Vec<Post>> {
    println!("reading markdown posts..");
    let mut options = ComrakOptions::default();
    options.unsafe_ = true;

    let files = fs::read_dir(format!("{}/posts", src_dir))?;
    let mut posts: Vec<Post> = Vec::new();

    for entry in files {
        let dir = entry?;
        let metadata = dir.metadata()?;

        if metadata.is_dir() {
            let mut meta_path = dir.path().clone();
            let mut markdown_path = dir.path().clone();

            meta_path.push("meta.json");
            markdown_path.push("content.md");

            let markdown = open_file(markdown_path.to_str().unwrap())?;
            let meta_json = open_file(meta_path.to_str().unwrap())?;

            let html = markdown_to_html(&markdown, &options);
            let slug = dir.file_name().to_str().unwrap().to_owned();

            posts.push(
                Post::new(
                    &slug,
                    &html,
                    &dir.path().to_str().unwrap(),
                    get_meta_from_json_with_default(&meta_json),
                )
            );
        }
    }

    Ok(posts)
}

fn get_template(src_dir: &str) -> std::io::Result<Template> {
    println!("reading template..");

    let html = open_file(&format!("{}/template.html", src_dir))?;
    let css = open_file(&format!("{}/style.css", src_dir))?;

    Ok(Template::new(&html, &css))
}

fn write_html_files(config: &Config, template: &Template, posts: &Vec<Post>) -> std::io::Result<()> {
    let path = Path::new(&config.dest);
    match fs::create_dir(path) {
        Ok(_) => println!("creating folder.."),
        Err(_) => {
            println!("destination folder already exists, cleaning up..");
            fs::remove_dir_all(path)?;
            println!("creating folder..");
            fs::create_dir(path)?;
        },
    }

    println!("creating posts..");
    for post in posts {
        let folder_path = path.clone().join(&post.slug);
        fs::create_dir(&folder_path)?;

        let file_path = folder_path.join("index.html");
        let mut file = File::create(file_path)?;

        let paths = fs::read_dir(&post.dir)?;

        for path in paths {
            let path = path?;
            let extension = path.path().extension().unwrap().to_owned();

            if extension == "png" || extension == "jpg" {
                let file_name = path.file_name().to_str().unwrap().to_owned();
                let new_path = folder_path.clone().join(file_name);
                fs::copy(path.path(), new_path)?;
            }
        }

        file.write_all(template.build_page(&config, &post.html, &post.title).as_bytes())?;
        println!("-> {}", &post.slug);
    }

    println!("creating index..");

    let index_path = path.clone().join("index.html");
    let mut file = File::create(index_path)?;
    let mut html = String::from("");

    // building up html like the good old days
    html.push_str("<ul>\n");
    for post in posts {
        html.push_str(&post.render_link());
    }
    html.push_str("</ul>\n");

    file.write_all(template.build_page(&config, &html, &config.title).as_bytes())?;

    println!("\ndone!");
    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Blog generator")
        .version("0.1")
        .arg(Arg::with_name("directory")
             .short("d")
             .long("directory")
             .help("directory containing blog config and files")
             .takes_value(true))
         .get_matches();

    let src_dir = matches.value_of("directory").unwrap_or("./blog");

    let config = get_config(&src_dir);
    let mut posts = get_posts(&src_dir)?;

    posts.sort_by(|a, b| {
        match (a.meta.timestamp, b.meta.timestamp) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => b.cmp(&a),
        }
    });

    let template = get_template(&src_dir)?;

    write_html_files(&config, &template, &posts)
}
