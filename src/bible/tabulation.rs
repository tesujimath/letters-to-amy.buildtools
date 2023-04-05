use super::Chapter;
use super::{books::Testament, AllReferences, ChapterVerses, ChaptersVerses, References};
use crate::hugo::{format_href, write_table, ContentWriter, Header, Metadata};
use crate::util::insert_in_order;
use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::{hash_map, HashMap},
    fmt::{self, Display, Formatter},
    io::{self, Write},
    ops::{Deref, DerefMut},
};

#[derive(PartialEq, Eq, Debug)]
// a post with just one chapters worth of references
pub struct PostReferences1 {
    pub post_index: usize,
    pub cv: ChapterVerses,
}

impl PostReferences1 {
    pub fn new(post_index: usize, cv: ChapterVerses) -> Self {
        Self { post_index, cv }
    }
}

impl PartialOrd for PostReferences1 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PostReferences1 {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self.cv.cmp(&other.cv) {
            Equal => self.post_index.cmp(&other.post_index),
            cmp => cmp,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
// a post with all its chapters' references
pub struct PostReferences {
    pub post_index: usize,
    pub cvs: ChaptersVerses,
}

impl PostReferences {
    fn from1(refs1: PostReferences1) -> Self {
        Self {
            post_index: refs1.post_index,
            cvs: ChaptersVerses::new(refs1.cv),
        }
    }

    fn push(&mut self, refs1: PostReferences1) {
        self.cvs.push(refs1.cv);
    }
}

impl Display for PostReferences {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#}", &self.cvs)
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

// separated references to a single book, non-empty
pub struct BookReferences1(Vec<PostReferences1>);

impl BookReferences1 {
    pub fn new(post_index: usize, cv: ChapterVerses) -> BookReferences1 {
        BookReferences1(vec![PostReferences1::new(post_index, cv)])
    }
}

impl Deref for BookReferences1 {
    type Target = Vec<PostReferences1>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BookReferences1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// all the references to a single book, non-empty
pub struct BookReferences(Vec<PostReferences>);

impl Deref for BookReferences {
    type Target = Vec<PostReferences>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// strategy for merging in a new reference
enum MergeStrategy {
    Append,
    Merge(usize),
    // TODO MoveAndMerge(usize, usize),
}

impl BookReferences {
    fn new(p: PostReferences) -> BookReferences {
        BookReferences(vec![p])
    }

    fn latest_same_post(&self, post_index: usize) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .rev()
            .find(|(_i, r)| r.post_index == post_index)
            .map(|(i, _r)| i)
    }

    fn earliest_insertion(&self, chapters: &[Chapter]) -> usize {
        self.0
            .iter()
            .enumerate()
            .rev()
            .take_while(|(_i, r)| r.cvs.chapters() == chapters)
            .map(|(i, _r)| i)
            .last()
            .unwrap_or(self.0.len())
    }

    fn merge_strategy(&self, r1: &PostReferences1) -> MergeStrategy {
        use MergeStrategy::*;

        let r1_chapters = r1.cv.chapters();

        if self.0.is_empty() {
            Append
        } else if let Some(i) = self.latest_same_post(r1.post_index) {
            let j = self.earliest_insertion(&r1_chapters);

            if j <= i + 1 {
                Merge(i)
            } else {
                Append
            }
        } else {
            Append
        }
    }

    fn from_separated(refs1: BookReferences1) -> BookReferences {
        let mut it1 = refs1.0.into_iter();
        let mut refs = BookReferences::new(PostReferences::from1(it1.by_ref().next().unwrap()));
        for r1 in it1 {
            use MergeStrategy::*;

            match refs.merge_strategy(&r1) {
                Append => refs.0.push(PostReferences::from1(r1)),
                Merge(i) => refs.0[i].push(r1),
            }
        }

        refs
    }
}

impl AllReferences {
    pub fn new() -> Self {
        AllReferences {
            metadata: Vec::new(),
            separated_refs_by_book: HashMap::new(),
            refs_by_book: HashMap::new(),
        }
    }

    // insert the post references separately and return a stable reference to its metadata
    pub fn insert(&mut self, metadata: Metadata, refs: References) -> &Metadata {
        self.metadata.push(metadata);
        let post_index = self.metadata.len() - 1;

        for (book, cvs) in refs.into_iter() {
            for cv in cvs.into_iter() {
                use hash_map::Entry::*;
                match self.separated_refs_by_book.entry(book) {
                    Occupied(mut o) => {
                        insert_in_order(o.get_mut(), PostReferences1::new(post_index, cv));
                    }
                    Vacant(v) => {
                        v.insert(BookReferences1::new(post_index, cv));
                    }
                }
            }
        }

        self.metadata.last().unwrap()
    }

    pub fn coelesce(&mut self) {
        self.refs_by_book = HashMap::<&str, BookReferences>::from_iter(
            self.separated_refs_by_book
                .drain()
                .map(|(k, v)| (k, BookReferences::from_separated(v))),
        );
    }

    pub fn dump_repeats(&self, mut w: impl Write) -> io::Result<()> {
        for testament in Testament::all() {
            w.write_all(format!("{}\n", testament.name()).as_bytes())?;

            for book in testament.books() {
                if let Some(refs) = self.refs_by_book.get(book) {
                    let mut post_count = HashMap::<usize, u8>::new();
                    for r in refs.iter() {
                        use hash_map::Entry::*;

                        match post_count.entry(r.post_index) {
                            Occupied(mut o) => {
                                *o.get_mut() += 1;
                            }
                            Vacant(v) => {
                                v.insert(1);
                            }
                        }
                    }

                    let mut written_book = false;

                    for (post_index, count) in post_count {
                        if count > 1 {
                            if !written_book {
                                w.write_all(format!("    {}\n", book).as_bytes())?;
                                written_book = true;
                            }
                            w.write_all(
                                format!(
                                    "            {} \"{}\"\n",
                                    count,
                                    self.metadata[post_index]
                                        .header
                                        .title
                                        .as_ref()
                                        .unwrap_or(&"UNTITLED".to_string())
                                )
                                .as_bytes(),
                            )?
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct Writer {
    w: ContentWriter,
}

impl Writer {
    pub fn new(w: ContentWriter) -> Self {
        Writer { w }
    }

    const BOOK_REFS_DESCRIPTION: &str = "Scripture index";

    fn write_book_refs(
        &mut self,
        book: &str,
        abbrev: &str,
        refs: &[PostReferences],
        posts: &AllReferences,
    ) -> anyhow::Result<String> {
        let h = Header::new(book, Self::BOOK_REFS_DESCRIPTION);
        self.w.create_leaf(&h).and_then(|(mut f, url)| {
            f.write_all("\n".as_bytes())?;

            let heading = vec!["", ""];
            let body = refs
                .iter()
                .map(|r| {
                    let m = &posts.metadata[r.post_index];
                    vec![r.to_string(), m.format_href()]
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
        posts: &AllReferences,
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

    pub fn write_references(&mut self, posts: &AllReferences) -> anyhow::Result<()> {
        self.w.create_branch().and_then(|f| {
            for testament in Testament::all() {
                let mut hrefs = Vec::new();

                self.write_refs(testament.books_with_abbrev(), &mut hrefs, posts)?;
                self.write_grid(&f, testament.name(), &hrefs)?;
            }

            Ok(())
        })
    }
}
