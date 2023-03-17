// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{ChaptersVerses, References};
use super::hugo::Metadata;
use super::util::insert_in_order;
use std::collections::hash_map;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(PartialEq, Eq, Debug)]
pub struct PostReferences {
    pub post_index: usize,
    pub cvs: ChaptersVerses,
}

impl PostReferences {
    fn new(post_index: usize, cvs: ChaptersVerses) -> Self {
        Self { post_index, cvs }
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

pub struct Posts {
    pub metadata: Vec<Metadata>,
    pub refs_by_book: HashMap<&'static str, Vec<PostReferences>>,
}

impl Posts {
    pub fn new() -> Self {
        Posts {
            metadata: Vec::new(),
            refs_by_book: HashMap::new(),
        }
    }

    pub fn insert(&mut self, metadata: Metadata, refs: References) {
        self.metadata.push(metadata);
        let post_index = self.metadata.len() - 1;

        for (book, cvs) in refs.into_iter() {
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
}
