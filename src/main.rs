mod config;
mod post;
mod template;

use clap::{App, Arg};
use config::Config;
use post::Post;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use template::Template;

fn write_html_files(
    src_path: &Path,
    config: &Config,
    template: &Template,
    posts: &[Post],
    static_pages: &[Post],
) -> std::io::Result<()> {
    let path = Path::new(&config.dest);
    match fs::create_dir(path) {
        Ok(_) => println!("creating folder.."),
        Err(_) => {
            println!("destination folder already exists, cleaning up..");
            fs::remove_dir_all(path)?;
            println!("creating folder..");
            fs::create_dir(path)?;
        }
    }

    println!("copying assets folder..");
    let assets_src = src_path.join("assets");
    let assets_dest = path.join("assets");

    fs::create_dir(&assets_dest)?;
    let assets = fs::read_dir(&assets_src)?;

    for asset in assets {
        let asset = asset?;
        let file_name = asset.file_name().to_str().unwrap().to_owned();
        let new_path = path.join("assets").clone().join(file_name);
        fs::copy(asset.path(), new_path)?;
    }

    println!("creating posts..");
    let mut tags_with_pages: HashMap<String, Vec<&Post>> = HashMap::new();

    for post in posts {
        if let Some(tags) = post.meta.tags.clone() {
            for tag in tags {
                match tags_with_pages.get_mut(&tag) {
                    Some(posts) => posts.push(post),
                    None => {
                        tags_with_pages.insert(tag, vec![post]);
                    }
                }
            }
        }

        let folder_path = path.join(&post.slug);
        fs::create_dir(&folder_path)?;

        let file_path = folder_path.join("index.html");
        let mut file = File::create(file_path)?;

        let paths = fs::read_dir(&post.dir)?;

        for path in paths {
            let path = path?;
            let extension = path.path().extension().unwrap().to_owned();

            if extension == "png" || extension == "jpg" || extension == "jpeg" {
                let file_name = path.file_name().to_str().unwrap().to_owned();
                let new_path = folder_path.clone().join(file_name);
                fs::copy(path.path(), new_path)?;
            }
        }

        file.write_all(
            template
                .build_page(config, &post.html, &post.title, Some(&post.meta))
                .as_bytes(),
        )?;
        println!("-> {}", &post.slug);
    }

    println!("creating static pages..");
    for post in static_pages {
        let folder_path = path.join(&post.slug);
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

        file.write_all(
            template
                .build_page(config, &post.html, &post.title, Some(&post.meta))
                .as_bytes(),
        )?;
        println!("-> {}", &post.slug);
    }

    println!("creating tag folder..");
    let tag_path = path.join("tags");
    fs::create_dir(&tag_path)?;

    println!("creating tag pages..");
    for (tag, p) in &tags_with_pages {
        let folder_path = tag_path.clone().join(&tag);
        let file_path = folder_path.clone().join("index.html");
        fs::create_dir(&folder_path)?;
        let mut file = File::create(file_path)?;
        let mut html = String::from("");

        // building up html like the good old days
        html.push_str(&format!("<h1>All posts tagged with {}</h1>\n", tag));
        html.push_str("<ul>\n");
        for post in p {
            html.push_str("<li>");
            html.push_str("<small>");
            html.push_str(&post.render_date());
            html.push_str("</small> ");
            html.push_str(&post.render_link());
            html.push_str("</li>\n");
        }
        html.push_str("</ul>\n");

        file.write_all(template.build_page(config, &html, tag, None).as_bytes())?;
        println!("-> {}", tag);
    }

    println!("creating index..");
    let index_path = path.join("index.html");
    let mut file = File::create(index_path)?;
    let mut html = String::from("");

    // building up html like the good old days
    html.push_str("<h2>Posts</h2>\n");
    html.push_str("<ul class=\"posts\">\n");
    for post in posts {
        html.push_str("<li>");
        html.push_str("<small>");
        html.push_str(&post.render_date());
        html.push_str("</small> ");
        html.push_str(&post.render_link());
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n");
    file.write_all(
        template
            .build_page(config, &html, &config.title, None)
            .as_bytes(),
    )?;

    println!("\ndone!");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Blog generator")
        .version("0.1.0")
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .help("directory containing blog config and files")
                .takes_value(true),
        )
        .get_matches();

    let src_dir = Path::new(matches.value_of("directory").unwrap_or("./blog"));

    println!("reading config..");

    let config: Config = File::open(src_dir.join("config.json"))
        .map_err(|_| "no config file found")?
        .try_into()
        .unwrap_or_else(|err| {
            println!("{}, using default one..", err);
            Default::default()
        });

    println!("reading markdown posts..");

    let files_posts =
        fs::read_dir(src_dir.join("posts/")).map_err(|_| "unable to find `/posts` folder")?;
    let files_static = fs::read_dir(src_dir.join("static/"));

    let mut posts: Vec<Post> = Vec::new();
    let mut static_pages: Vec<Post> = Vec::new();

    for entry in files_posts {
        let dir = entry.map_err(|_| "cant read entry")?;
        let metadata = dir.metadata().map_err(|_| "cant read metadata")?;

        if metadata.is_dir() {
            let post: Post = dir
                .path()
                .to_str()
                .unwrap()
                .to_owned()
                .try_into()
                .map_err(|_| "cant parse folder into post")?;

            posts.push(post);
        }
    }

    if let Ok(files) = files_static {
        for entry in files {
            let dir = entry.map_err(|_| "cant read entry")?;
            let metadata = dir.metadata().map_err(|_| "cant read metadata")?;

            if metadata.is_dir() {
                let post: Post = dir
                    .path()
                    .to_str()
                    .unwrap()
                    .to_owned()
                    .try_into()
                    .map_err(|_| "cant parse folder into post")?;

                static_pages.push(post);
            }
        }
    }

    posts.sort_by(|a, b| match (a.meta.timestamp, b.meta.timestamp) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => b.cmp(&a),
    });

    println!("reading template..");
    let template: Template = src_dir
        .to_str()
        .unwrap()
        .to_owned()
        .try_into()
        .map_err(|_| "cant create template")?;

    write_html_files(&src_dir, &config, &template, &posts, &static_pages).map_err(|e| e.into())
}
