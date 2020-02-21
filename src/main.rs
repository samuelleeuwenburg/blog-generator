use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use regex::Regex;
use comrak::{markdown_to_html, ComrakOptions};

struct Config {
    src: String,
    dest: String,
    title: String,
    selector_content: String,
    selector_title: String,
    selector_css: String,
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
}

impl Post {
    fn new(slug: &str, html: &str, dir: &str) -> Post {
        let re = Regex::new("<h1>(.+)</h1>").unwrap();

        let title = match re.captures(html) {
            Some(capture) => {
                String::from(capture.get(0).unwrap().as_str())
            }
            None => String::from(slug),
        };

        println!("{}", dir);

        Post {
            title,
            slug: String::from(slug),
            html: String::from(html),
            dir: String::from(dir),
        }
    }

    fn render_link(&self) -> String {
        format!("\n<a href=\"/{}\">{}</a>", self.slug, self.title)
    }
}

fn open_file(path: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;

    Ok(content)
}

fn get_posts(config: &Config) -> std::io::Result<Vec<Post>> {
    println!("reading markdown posts..");

    let files = fs::read_dir(format!("{}/posts", config.src))?;
    let mut posts: Vec<Post> = Vec::new();

    for entry in files {
        let dir = entry?;
        let metadata = dir.metadata()?;

        if metadata.is_dir() {
            let mut path = dir.path().clone();
            path.push("content.md");

            let markdown = open_file(path.to_str().unwrap())?;
            let html = markdown_to_html(&markdown, &ComrakOptions::default());
            let slug = dir.file_name().to_str().unwrap().to_owned();

            posts.push(Post::new(&slug, &html, &dir.path().to_str().unwrap()));
        }
    }

    Ok(posts)
}

fn get_template(config: &Config) -> std::io::Result<Template> {
    println!("reading template..");

    let html = open_file(&format!("{}/template.html", config.src))?;
    let css = open_file(&format!("{}/style.css", config.src))?;

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

    for post in posts {
        html.push_str(&post.render_link());
    }

    file.write_all(template.build_page(&config, &html, &config.title).as_bytes())?;

    println!("\ndone!");
    Ok(())
}

fn main() -> std::io::Result<()> {
    let config = Config {
        src: String::from("./blog"),
        dest: String::from("./dist"),
        title: String::from("Blogname"),
        selector_content: String::from("CONTENT"),
        selector_title: String::from("TITLE"),
        selector_css: String::from("CSS"),
    };

    let posts = get_posts(&config)?;
    let template = get_template(&config)?;

    write_html_files(&config, &template, &posts)
}
