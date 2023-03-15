// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{ChaptersVerses, References};
use super::hugo::Metadata;
use std::collections::hash_map;
use std::{cmp::Ordering, collections::HashMap};

#[derive(PartialEq, Eq, Debug)]
pub struct PostReferences<'a> {
    metadata: &'a Metadata,
    cvs: &'a ChaptersVerses,
}

impl<'a> PostReferences<'a> {
    fn new(metadata: &'a Metadata, cvs: &'a ChaptersVerses) -> Self {
        Self { metadata, cvs }
    }
}

impl<'a> PartialOrd for PostReferences<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for PostReferences<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cvs.cmp(&other.cvs)
    }
}

pub struct Posts<'a> {
    refs: Vec<PostReferences<'a>>,
    refs_by_book: HashMap<&'static str, Vec<PostReferences<'a>>>,
}

impl<'a> Posts<'a> {
    pub fn new() -> Self {
        Posts {
            refs: Vec::new(),
            refs_by_book: HashMap::new(),
        }
    }

    pub fn insert<'b>(&mut self, metadata: &'b Metadata, refs: &'b References)
    where
        'b: 'a,
    {
        println!(
            "-------------------- {} '{}'",
            metadata.url, metadata.header.title
        );

        for (book, cvs) in refs.iter() {
            println!("{} {}", book, &cvs);
            let post_refs = PostReferences::new(metadata, cvs);

            self.refs.push(post_refs);

            // use hash_map::Entry::*;
            // match self.refs_by_book.entry(book) {
            //     Occupied(mut o) => {
            //         let refs = o.get_mut();
            //         // TODO insert in order
            //         refs.push(post_refs);
            //     }
            //     Vacant(v) => {
            //         v.insert(vec![post_refs]);
            //     }
            // }
        }
    }
}
