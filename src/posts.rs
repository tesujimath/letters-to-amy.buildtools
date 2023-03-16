// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use itertools::Itertools;

use super::bible::{ChaptersVerses, References};
use super::hugo::Metadata;
use super::util::insert_in_order;
use std::collections::hash_map;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(PartialEq, Eq, Debug)]
pub struct PostReferences {
    post_index: usize,
    cvs: ChaptersVerses,
}

impl PostReferences {
    fn new(post_index: usize, cvs: ChaptersVerses) -> Self {
        Self { post_index, cvs }
    }
}

impl Display for PostReferences {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.cvs)
    }
}

impl PartialOrd for PostReferences {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PostReferences {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self.cvs.cmp(&other.cvs) {
            Equal => self.post_index.cmp(&other.post_index),
            cmp => cmp,
        }
    }
}

pub struct Posts {
    metadata: Vec<Metadata>,
    refs: Vec<References>,
    refs_by_book: HashMap<&'static str, Vec<PostReferences>>,
}

impl Posts {
    pub fn new() -> Self {
        Posts {
            metadata: Vec::new(),
            refs: Vec::new(),
            refs_by_book: HashMap::new(),
        }
    }

    pub fn insert(&mut self, metadata: Metadata, refs: References) {
        println!(
            "-------------------- {} '{}'",
            &metadata.url, &metadata.header.title
        );

        self.metadata.push(metadata);
        let post_index = self.metadata.len() - 1;

        for (book, cvs) in refs.into_iter() {
            println!("{} {}", book, &cvs);
            let post_refs = PostReferences::new(post_index, cvs);

            use hash_map::Entry::*;
            match self.refs_by_book.entry(book) {
                Occupied(mut o) => {
                    insert_in_order(o.get_mut(), post_refs);
                }
                Vacant(v) => {
                    v.insert(vec![post_refs]);
                }
            }
        }
    }

    fn write_book_refs(
        &self,
        section_dir: &PathBuf,
        section_name: &str,
        book: &str,
        refs: &Vec<PostReferences>,
    ) -> anyhow::Result<String> {
        let slug = slug::slugify(book);
        let path = section_dir.join(format!("{}.md", slug));

        let mut f = File::create(path).unwrap();
        f.write_all(format!("---\ntitle: \"{}\"\n---\n\n| | |\n| --- | --- |\n", book).as_bytes());

        for r in refs {
            let m = &self.metadata[r.post_index];
            f.write_all(
                format!(
                    "| [{}]({{{{<ref \"/post/{}\" >}}}}) | {} |\n",
                    &m.header.title, &m.url, r
                )
                .as_bytes(),
            );
        }

        let abbrev = super::bible::abbrev(book).unwrap_or(book);
        let href = format!(
            "[{}]({{{{<ref \"/{}/{}\" >}}}})",
            abbrev, section_name, slug
        );

        Ok(href)
    }

    pub fn dump(
        &self,
        page_header: &str,
        mut outfile: impl Write,
        section_dir: &PathBuf,
        section_name: &str,
    ) {
        outfile.write_all(format!("---\n{}---\n", page_header).as_bytes());

        let mut hrefs = Vec::new();
        for book in super::bible::books() {
            if let Some(refs) = self.refs_by_book.get(book) {
                let href = self
                    .write_book_refs(section_dir, section_name, book, refs)
                    .unwrap();
                hrefs.push(href);
            }
        }

        const row_size: usize = 4;
        for i in 0..row_size {
            outfile.write_all("| ".as_bytes());
        }
        outfile.write_all("|\n".as_bytes());
        for i in 0..row_size {
            outfile.write_all("| --- ".as_bytes());
        }
        outfile.write_all("|\n".as_bytes());

        for href_batch in &hrefs.into_iter().chunks(row_size) {
            for href in href_batch {
                outfile.write_all(format!("| {} ", href).as_bytes());
            }
            outfile.write_all("|\n".as_bytes());
        }
    }
}
