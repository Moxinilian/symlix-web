use anyhow::{Context, Result};
use itertools::Itertools;
use tera::Tera;

use crate::{
    data::{MusicDB, Stream, StreamDB},
    Args,
};

const STREAMS_PER_PAGE: usize = 8;

pub fn generate_music_pages(
    args: &Args,
    templates: &Tera,
    ctx: &tera::Context,
    streams: &StreamDB,
    music: &MusicDB,
) -> Result<()> {
    let mut ctx = ctx.clone();

    let music_dir = args.output.join("music");
    std::fs::create_dir(&music_dir).context("failed to create music pages directory")?;

    ctx.insert("music", music);

    let pages_to_generate: Vec<Vec<&Stream>> = streams
        .iter()
        .rev()
        .chunks(STREAMS_PER_PAGE)
        .into_iter()
        .map(Iterator::collect::<Vec<&Stream>>)
        .collect();

    for (page, streams) in pages_to_generate.iter().enumerate() {
        let page = page + 1;
        let page_directory = music_dir.join(page.to_string());
        std::fs::create_dir(&page_directory).context("failed to create music page directory")?;

        ctx.insert("streams", &streams);
        ctx.insert("page", &page);

        ctx.insert("is_first_page", &(page == 1));
        ctx.insert("is_last_page", &(page == pages_to_generate.len()));

        let rendered = templates.render("music.html", &ctx)?;

        let minified =
            minify_html::minify(rendered.as_bytes(), &minify_html::Cfg::spec_compliant());
        std::fs::write(page_directory.join("index.html"), minified)
            .context("failed to write music page template result")?;

        if page == 1 {
            std::fs::copy(
                page_directory.join("index.html"),
                music_dir.join("index.html"),
            )
            .context("failed to copy first music page to main folder")?;
        }
    }

    Ok(())
}
