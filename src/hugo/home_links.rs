// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::docs::Docs;
use anyhow::{Context, Result};
use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn contextualize(docs: &Docs) -> Result<()> {
    for p in docs.pages()?.filter_map(|p| p.ok()) {
        let (page_number, path) = p;

        let page_href = format!("/page/{}/", page_number);

        println!("{}: {:?}", page_number, &path);

        let page_post_hrefs = get_posts_for_page(&path)?;

        for page_post_href in page_post_hrefs {
            let src_path = docs.index_path(&page_post_href);
            let dst_path = PathBuf::from(format!("{}.new", src_path.display()));
            rewrite_home_link(src_path, dst_path, &page_href)?;
        }
    }

    Ok(())
}

fn get_posts_for_page<P>(page_index_path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let mut f = File::open(&page_index_path)
        .context(format!("open(\"{}\")", page_index_path.as_ref().display()))?;
    let mut buffer = Vec::new();
    let mut post_hrefs = Vec::new();

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // Rewrite insecure hyperlinks
                element!("h2.article-title a[href]", |el| {
                    let href = el.get_attribute("href").unwrap(); //.replace("http:", "https:");
                    post_hrefs.push(href);
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |_c: &[u8]| (), // don't write anything
    );

    f.read_to_end(&mut buffer)?;

    rewriter.write(&buffer)?;
    drop(rewriter);

    Ok(post_hrefs)
}

fn rewrite_home_link<P1, P2>(src_path: P1, dst_path: P2, page_href: &str) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut f_src =
        File::open(&src_path).context(format!("open(\"{}\")", src_path.as_ref().display()))?;
    let mut f_dst =
        File::create(&dst_path).context(format!("create(\"{}\")", dst_path.as_ref().display()))?;

    let mut src_buf = Vec::new();
    let mut dst_buf = Vec::new();

    let href_css_selectors = ["figure.site-avatar", "ol.menu li"];

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: href_css_selectors
                .iter()
                .map(|css| {
                    element!(format!("{} a[href]", css), |el| {
                        if let Some(existing_href) = el.get_attribute("href") {
                            if existing_href == "/" {
                                el.set_attribute("href", page_href)?
                            }
                        }
                        Ok(())
                    })
                })
                .collect(),
            ..Settings::default()
        },
        |c: &[u8]| dst_buf.extend_from_slice(c), // don't write anything
    );

    f_src.read_to_end(&mut src_buf)?;

    rewriter.write(&src_buf)?;
    f_dst.write_all(&dst_buf)?;

    Ok(())
}
