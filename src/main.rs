use std::{fs::File, path::PathBuf, time::Instant};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use data::{MusicDB, StreamDB};
use music::generate_music_pages;
use pages::generate_custom_pages;
use serde::Deserialize;
use walkdir::WalkDir;

mod data;
mod music;
mod pages;

#[cfg(feature = "dev")]
mod serve;

#[derive(Deserialize)]
struct Config {
    twitch: String,
    youtube: String,
    discord: String,
    base_url: String,
}

/// Basic static site generator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Templates folder
    #[clap(long, value_parser, default_value = "templates")]
    templates: PathBuf,

    /// Data folder
    #[clap(long, value_parser, default_value = "data")]
    data: PathBuf,

    /// Static folder
    #[clap(long, value_parser, default_value = "static")]
    r#static: PathBuf,

    /// Output folder
    #[clap(short, long, value_parser, default_value = "dist")]
    output: PathBuf,

    #[clap(short, long, value_parser, default_value = "config.toml")]
    /// Config file
    config: PathBuf,

    /// Serve and regenerate the website upon file changes
    #[cfg(feature = "dev")]
    #[clap(short, long)]
    serve: bool,

    /// Server address
    #[cfg(feature = "dev")]
    #[clap(short, long, value_parser, default_value = "127.0.0.1:8080")]
    addr: String,

    /// Hot reloading server address
    #[cfg(feature = "dev")]
    #[clap(short, long, value_parser, default_value = "127.0.0.1:9595")]
    hrs_addr: String,

    /// Allow caching
    #[cfg(feature = "dev")]
    #[clap(long, value_parser)]
    allow_caching: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Err(e) = generate(&args) {
        eprintln!("\x1b[31m[ERROR] {:#}\x1b[0m", e);
    }

    #[cfg(feature = "dev")]
    if args.serve {
        serve::serve(args)?;
    }

    Ok(())
}

pub fn generate(args: &Args) -> Result<()> {
    let config: Config = toml::from_str(
        &std::fs::read_to_string(&args.config)
            .context("failed to read config file")
            .context("failed to deserialize config file")?,
    )?;

    let start = Instant::now();
    if args.output.exists() {
        if args.output.is_dir() {
            for entry in
                std::fs::read_dir(&args.output).context("failed to read directory to delete")?
            {
                let entry = entry.context("failed to read directory entry to delete")?;
                if entry
                    .metadata()
                    .context("failed to get metadata of entry to delete")?
                    .is_dir()
                {
                    std::fs::remove_dir_all(entry.path())
                        .context("failed to delete entry of directory kind")?;
                } else {
                    std::fs::remove_file(entry.path())
                        .context("failed to delete entry of file kind")?;
                }
            }
        } else {
            return Err(anyhow!(
                "output path {} is not a directory",
                args.output.display()
            ));
        }
    } else {
        std::fs::create_dir(&args.output).expect("failed to create output directory");
    }

    for p in WalkDir::new(&args.r#static).into_iter().skip(1) {
        let p = p.context("failed to read static asset entry")?;
        let path = p
            .path()
            .strip_prefix(&args.r#static)
            .expect("unreachable: walkdir preserves root");

        if p.metadata()
            .context("failed to read static asset metadata")?
            .is_dir()
        {
            std::fs::create_dir(args.output.join(path))
                .context("failed to create static asset directory")?;
        } else {
            match path.extension().map(|ext| ext.to_string_lossy()).as_deref() {
                Some("html") => {
                    let file = std::fs::read(p.path()).context("failed to read html file")?;

                    let minified = minify_html::minify(&file, &minify_html::Cfg::spec_compliant());
                    std::fs::write(args.output.join(path), minified)
                        .context("failed to write minified html file")?;
                }
                Some("css") => {
                    let file =
                        std::fs::read_to_string(p.path()).context("failed to read css file")?;
                    let minified = css_minify::optimizations::Minifier::default()
                        .minify(&file, css_minify::optimizations::Level::One)
                        .map_err(|e| anyhow!("failed to minify css: {}", e))?;
                    std::fs::write(args.output.join(path), minified)
                        .context("failed to write minified css")?;
                }
                Some("js") => {
                    let file = std::fs::read(p.path()).context("failed to read js file")?;
                    let mut output_file = File::create(args.output.join(path))
                        .context("failed to create file for minified js")?;
                    minify_js::minify(file, &mut output_file)
                        .map_err(|e| anyhow!("failed to minify js: {:?}", e))?;
                }
                Some(_) | None => {
                    std::fs::copy(p.path(), args.output.join(path))
                        .context("failed to copy static asset")?;
                }
            }
        }
    }

    let mut templates = tera::Tera::default();

    let mut templates_to_load = Vec::new();
    for p in WalkDir::new(&args.templates) {
        let p = p.context("failed to find template file")?;
        if !p
            .metadata()
            .context("failed to find template metadata")?
            .is_dir()
        {
            templates_to_load.push((
                p.path().to_owned(),
                Some(
                    p.path()
                        .strip_prefix(&args.templates)
                        .expect("unreachable: walkdir preserves root")
                        .to_string_lossy()
                        .to_string(),
                ),
            ));
        }
    }

    templates
        .add_template_files(templates_to_load)
        .context("failed to load templates")?;

    let mut ctx = tera::Context::new();

    ctx.insert("twitch", &config.twitch);
    ctx.insert("youtube", &config.youtube);
    ctx.insert("discord", &config.discord);
    ctx.insert("dev", "");

    #[cfg(feature = "dev")]
    if args.serve {
        ctx.insert("dev", &serve::get_dev_html_insert(args)?);
    }

    let streams: StreamDB = ron::de::from_reader(
        &File::open(&args.data.join("streams.ron")).context("failed to open stream database")?,
    )
    .context("failed to load stream database")?;

    let music: MusicDB = ron::de::from_reader(
        &File::open(&args.data.join("music.ron")).context("failed to open music database")?,
    )
    .context("failed to load music database")?;

    ctx.insert("stream_amount", &streams.len());
    ctx.insert("music_amount", &music.len());

    ctx.insert("url", &config.base_url);
    let rendered = templates.render("base.html", &ctx)?;
    ctx.remove("url");

    let minified = minify_html::minify(rendered.as_bytes(), &minify_html::Cfg::spec_compliant());
    std::fs::write(&args.output.join("index.html"), minified)
        .context("failed to write index template result")?;

    generate_custom_pages(&config.base_url, args, &templates, ctx.clone())?;
    generate_music_pages(&config.base_url, args, &templates, &ctx, &streams, &music)?;

    let elapsed = start.elapsed();

    if elapsed.as_secs() > 0 {
        println!(
            "[INFO] \x1b[1mGenerated in {}.{}s\x1b[0m",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
    } else {
        println!(
            "[INFO] \x1b[1mGenerated in {}ms\x1b[0m",
            elapsed.as_millis() + 1
        );
    }

    Ok(())
}
