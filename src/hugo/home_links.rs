use super::docs::Docs;
use anyhow::{Context, Result};
use lol_html::{element, HtmlRewriter, Settings};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn contextualize(docs: &Docs) -> Result<()> {
    for p in docs.pages()?.filter_map(|p| p.ok()) {
        let (page_number, path) = p;

        let page_href = format!("/page/{}/", page_number);
        let post_hrefs = get_posts_for_page(&path)?;

        for post_href in post_hrefs {
            let post_path = docs.index_path(&post_href);
            rewrite_home_link(post_path, &page_href)?;
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
            element_content_handlers: vec![element!("h2.article-title a[href]", |el| {
                let href = el.get_attribute("href").unwrap();
                post_hrefs.push(href);
                Ok(())
            })],
            ..Settings::default()
        },
        |_c: &[u8]| (), // don't write anything, we're just snooping around
    );

    f.read_to_end(&mut buffer)?;

    rewriter.write(&buffer)?;
    drop(rewriter);

    Ok(post_hrefs)
}

fn rewrite_home_link<P>(post_path: P, page_href: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut src_buf = Vec::new();
    let mut dst_buf = Vec::new();

    let href_css_selectors = [
        "figure.site-avatar",
        "div.site-meta h1.site-name",
        // leave the memu pointing at home for now
        //"ol.menu li"
    ];

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
        |c: &[u8]| dst_buf.extend_from_slice(c),
    );

    let mut f =
        File::open(&post_path).context(format!("open(\"{}\")", post_path.as_ref().display()))?;
    f.read_to_end(&mut src_buf)?;

    rewriter.write(&src_buf)?;

    f = File::create(&post_path)
        .context(format!("create(\"{}\")", post_path.as_ref().display()))?;

    f.write_all(&dst_buf)?;

    Ok(())
}
