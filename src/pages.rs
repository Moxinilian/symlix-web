use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use tera::Tera;
use walkdir::WalkDir;

use crate::Args;

pub fn generate_custom_pages(
    base_url: &str,
    args: &Args,
    templates: &Tera,
    mut ctx: tera::Context,
) -> Result<()> {
    let empty_path = PathBuf::default();

    for page in WalkDir::new(args.templates.join("pages"))
        .into_iter()
        .skip(1)
    {
        let page = page.context("failed to find page template file")?;
        let path = page
            .path()
            .strip_prefix(&args.templates.join("pages"))
            .expect("unreachable: walkdir preserves root");
        let template_path = page
            .path()
            .strip_prefix(&args.templates)
            .expect("unreachable: walkdir preserves root");
        if page
            .metadata()
            .context("failed to find page template metadata")?
            .is_dir()
        {
            std::fs::create_dir(args.output.join(path)).with_context(|| {
                format!(
                    "failed to create output folder for pages at {}",
                    args.output.join(path).display()
                )
            })?;
        } else if let Some(name) = path.file_stem() {
            let folder = path.parent().unwrap_or(&empty_path);
            let output_folder = folder.join(name);
            std::fs::create_dir(args.output.join(&output_folder))
                .context("failed to create page output folder")?;
            let index = output_folder.join("index.html");

            ctx.insert("url", &[base_url, "/", &output_folder.to_string_lossy()].concat());
            let rendered = templates.render(
                template_path
                    .to_str()
                    .ok_or_else(|| anyhow!("template file path is not valid UTF-8"))?,
                &ctx,
            )?;
            ctx.remove("url");

            let minified =
                minify_html::minify(rendered.as_bytes(), &minify_html::Cfg::spec_compliant());
            std::fs::write(&args.output.join(index), minified)
                .context("failed to write page template result")?;
        }
    }

    Ok(())
}
