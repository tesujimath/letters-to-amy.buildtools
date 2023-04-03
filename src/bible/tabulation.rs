use super::books::{nt_books_with_abbrev, ot_books_with_abbrev};
use crate::hugo::{format_href, write_table, ContentWriter, Header};
use crate::posts::{PostReferences, Posts};
use itertools::Itertools;
use std::io::Write;

pub struct ScriptureIndexWriter {
    w: ContentWriter,
}

impl ScriptureIndexWriter {
    pub fn new(w: ContentWriter) -> Self {
        ScriptureIndexWriter { w }
    }

    const BOOK_REFS_DESCRIPTION: &str = "Scripture index";

    fn write_book_refs(
        &mut self,
        book: &str,
        abbrev: &str,
        refs: &Vec<PostReferences>,
        posts: &Posts,
    ) -> anyhow::Result<String> {
        let h = Header::new(book, Self::BOOK_REFS_DESCRIPTION);
        self.w.create_leaf(&h).and_then(|(mut f, url)| {
            f.write_all("\n".as_bytes())?;

            let heading = vec!["", ""];
            let body = refs
                .iter()
                .map(|r| {
                    let m = &posts.metadata[r.post_index];
                    vec![m.format_href(), r.to_string()]
                })
                .collect::<Vec<Vec<String>>>();

            write_table(&f, heading, body)?;

            f.flush()?;
            Ok(format_href(abbrev, &url))
        })
    }

    fn write_refs(
        &mut self,
        book_abbrev_iter: impl Iterator<Item = (&'static str, &'static str)>,
        hrefs: &mut Vec<String>,
        posts: &Posts,
    ) -> anyhow::Result<()> {
        for (book, abbrev) in book_abbrev_iter {
            if let Some(refs) = posts.refs_by_book.get(book) {
                let href = self.write_book_refs(book, abbrev, refs, posts)?;
                hrefs.push(href);
            }
        }
        Ok(())
    }

    fn write_grid(
        &mut self,
        mut f: impl Write,
        heading: &str,
        hrefs: &[String],
    ) -> anyhow::Result<()> {
        f.write_all(format!("\n**{}**\n", heading).as_bytes())?;

        const ROW_SIZE: usize = 4;
        let header = std::iter::repeat("").take(ROW_SIZE);

        write_table(&mut f, header, &hrefs.iter().chunks(ROW_SIZE))?;

        Ok(())
    }

    pub fn write_posts(&mut self, posts: &Posts) -> anyhow::Result<()> {
        self.w.create_branch().and_then(|f| {
            let mut ot_hrefs = Vec::new();
            let mut nt_hrefs = Vec::new();

            self.write_refs(ot_books_with_abbrev(), &mut ot_hrefs, posts)?;
            self.write_grid(&f, "Old Testament", &ot_hrefs)?;

            self.write_refs(nt_books_with_abbrev(), &mut nt_hrefs, posts)?;
            self.write_grid(&f, "New Testament", &nt_hrefs)?;

            Ok(())
        })
    }
}
