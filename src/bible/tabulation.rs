use super::{books::Testament, AllReferences, ChapterVerses, ChaptersVerses, References};
use crate::hugo::{format_href, write_table, Create, Header, Metadata};
use crate::util::insert_in_order;
use anyhow::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::{
    cmp::Ordering,
    collections::{hash_map, HashMap},
    fmt::{self, Display, Formatter},
    io::{self, Write},
};
use time::{format_description::well_known::iso8601::Iso8601, OffsetDateTime};

#[derive(PartialEq, Eq, Clone, Debug)]
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

#[derive(PartialEq, Eq, Clone, Debug)]
// a post with all its chapters' references
pub struct PostReferences {
    pub post_index: usize,
    pub cvs: ChaptersVerses,
}

impl PostReferences {
    fn push(&mut self, refs1: PostReferences1) {
        self.cvs.0.push(refs1.cv);
    }
}

impl From<PostReferences1> for PostReferences {
    fn from(refs1: PostReferences1) -> Self {
        Self {
            post_index: refs1.post_index,
            cvs: ChaptersVerses::new(refs1.cv),
        }
    }
}

impl Display for PostReferences {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#}", &self.cvs)
    }
}

impl PartialOrd for PostReferences {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        match self.cvs.partial_cmp(&other.cvs) {
            Some(Equal) => Some(self.post_index.cmp(&other.post_index)),
            cmp => cmp,
        }
    }
}

// separated references to a single book, non-empty
#[derive(Debug)]
pub struct BookReferences1(Vec<PostReferences1>);

impl BookReferences1 {
    pub fn new(post_index: usize, cv: ChapterVerses) -> BookReferences1 {
        BookReferences1(vec![PostReferences1::new(post_index, cv)])
    }
}

// all the references to a single book, non-empty
#[derive(Debug)]
pub struct BookReferences(Vec<PostReferences>);

// strategy for merging in a new reference
enum MergeStrategy {
    Append,
    MoveAndMerge(usize, usize),
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

    fn merge_strategy(&self, r1: &PostReferences1) -> MergeStrategy {
        use MergeStrategy::*;

        match r1.cv.chapter {
            // don't need to merge in books without chapters
            None => Append,
            Some(_) => {
                if let Some(i_same_post) = self.latest_same_post(r1.post_index) {
                    // see if we can maintain order by merging these

                    // make a temporary candidate and test that
                    let mut candidate = self.0[i_same_post].clone();
                    candidate.push(r1.clone());

                    let orderings = self.0[i_same_post + 1..]
                        .iter()
                        .map(|p| candidate.partial_cmp(p))
                        .collect::<Vec<Option<Ordering>>>();

                    if orderings.iter().all(|o| o.is_some()) {
                        // can merge, yay!
                        // so find where we have to move the previous post so we can merge
                        match orderings
                            .iter()
                            .zip(i_same_post + 1..)
                            .find(|(o, _)| **o == Some(Ordering::Less))
                        {
                            Some((_, i)) => MoveAndMerge(i_same_post, i),
                            None => MoveAndMerge(i_same_post, self.0.len()),
                        }
                    } else {
                        // nope
                        Append
                    }
                } else {
                    Append
                }
            }
        }
    }

    fn from_separated(refs1: BookReferences1) -> BookReferences {
        let mut it1 = refs1.0.into_iter();
        let mut refs = BookReferences::new(PostReferences::from(it1.by_ref().next().unwrap())); // never empty
        for r1 in it1 {
            use MergeStrategy::*;

            match refs.merge_strategy(&r1) {
                Append => refs.0.push(PostReferences::from(r1)),
                MoveAndMerge(i_src, i_dst) => {
                    let p = refs.0.remove(i_src);
                    refs.0.insert(i_dst - 1, p);
                    refs.0[i_dst - 1].push(r1);
                }
            }
        }

        refs
    }
}

impl AllReferences {
    pub fn new() -> Self {
        AllReferences {
            metadata: Vec::new(),
            post_index_by_epoch: BTreeMap::new(),
            post_sequence_number_by_index: Vec::new(),
            separated_refs_by_book: HashMap::new(),
            refs_by_book: HashMap::new(),
        }
    }

    pub fn tabulate(&mut self, c: Box<dyn Create>, with_sequence_numbers: bool) -> Result<()> {
        if with_sequence_numbers {
            self.calculate_post_sequence_numbers();
        }
        self.coelesce();
        // useful for diagnostics:
        //self.dump_repeats(io::stdout())?;

        let mut w = Writer::new(c);
        w.write_references(self, with_sequence_numbers)?;

        Ok(())
    }

    // insert the post references separately and return a stable reference to its metadata
    pub fn insert(&mut self, metadata: Metadata, refs: References) -> &Metadata {
        self.metadata.push(metadata);
        self.post_sequence_number_by_index.push(None);

        let post_index = self.metadata.len() - 1;
        let post_header = &self.metadata.last().unwrap().header;

        if let Some(epoch) = post_header.date.as_ref().and_then(|date| {
            OffsetDateTime::parse(date.as_str(), &Iso8601::DEFAULT)
                .ok()
                .map(|date| date.unix_timestamp())
        }) {
            self.post_index_by_epoch.insert(epoch, post_index);
        }

        for (book, cvs) in refs.into_iter() {
            for cv in cvs.into_iter() {
                use hash_map::Entry::*;
                match self.separated_refs_by_book.entry(book) {
                    Occupied(mut o) => {
                        let br = o.get_mut();
                        insert_in_order(&mut br.0, PostReferences1::new(post_index, cv));
                    }
                    Vacant(v) => {
                        v.insert(BookReferences1::new(post_index, cv));
                    }
                }
            }
        }

        self.metadata.last().unwrap() // always exists
    }

    fn calculate_post_sequence_numbers(&mut self) {
        for (i, (_, post_index)) in self.post_index_by_epoch.iter().enumerate() {
            self.post_sequence_number_by_index[*post_index] = Some(i + 1);
        }
    }

    fn coelesce(&mut self) {
        self.refs_by_book = HashMap::<&str, BookReferences>::from_iter(
            self.separated_refs_by_book
                .drain()
                .map(|(k, v)| (k, BookReferences::from_separated(v))),
        );
    }

    pub fn _dump_repeats(&self, mut w: impl Write) -> io::Result<()> {
        for testament in Testament::all() {
            w.write_all(format!("{}\n", testament.name()).as_bytes())?;

            for book in testament._books() {
                if let Some(refs) = self.refs_by_book.get(book) {
                    let mut post_count = HashMap::<usize, u8>::new();
                    for r in refs.0.iter() {
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
    c: Box<dyn Create>,
}

impl Writer {
    pub fn new(c: Box<dyn Create>) -> Self {
        Writer { c }
    }

    const BOOK_REFS_DESCRIPTION: &str = "Scripture index";

    fn write_book_refs(
        &mut self,
        book: &str,
        abbrev: &str,
        refs: &[PostReferences],
        posts: &AllReferences,
        with_sequence_numbers: bool,
    ) -> anyhow::Result<String> {
        let h = Header::new(book, Self::BOOK_REFS_DESCRIPTION);
        self.c.create_leaf(&h).and_then(|(mut f, url)| {
            f.write_all("\n".as_bytes())?;

            let heading = vec!["", ""];
            let body = refs
                .iter()
                .map(|r| {
                    let m = &posts.metadata[r.post_index];
                    let sequence_number = &posts.post_sequence_number_by_index[r.post_index];
                    vec![
                        r.to_string(),
                        m.format_href(if with_sequence_numbers {
                            sequence_number
                        } else {
                            &None
                        }),
                    ]
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
        with_sequence_numbers: bool,
    ) -> anyhow::Result<()> {
        for (book, abbrev) in book_abbrev_iter {
            if let Some(refs) = posts.refs_by_book.get(book) {
                let href =
                    self.write_book_refs(book, abbrev, &refs.0, posts, with_sequence_numbers)?;
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

    pub fn write_references(
        &mut self,
        posts: &AllReferences,
        with_sequence_numbers: bool,
    ) -> anyhow::Result<()> {
        self.c.create_branch().and_then(|f| {
            for testament in Testament::all() {
                let mut hrefs = Vec::new();

                self.write_refs(
                    testament.books_with_abbrev(),
                    &mut hrefs,
                    posts,
                    with_sequence_numbers,
                )?;
                self.write_grid(&f, testament.name(), &hrefs)?;
            }

            Ok(())
        })
    }
}

mod tests;
