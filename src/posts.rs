// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{ChaptersVerses, References};
use super::hugo::Metadata;
use std::{cmp::Ordering, collections::HashMap};

#[derive(PartialEq, Eq, Debug)]
pub struct PostReferences<'a> {
    metadata: &'a Metadata,
    cvs: ChaptersVerses,
}

impl<'a> PostReferences<'a> {
    fn new(metadata: &'a Metadata, cvs: ChaptersVerses) -> Self {
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

pub struct AllPostsReferences<'a> {
    refs_by_book: HashMap<&'static str, Vec<PostReferences<'a>>>,
}

impl<'a> AllPostsReferences<'a> {
    pub fn new() -> Self {
        AllPostsReferences {
            refs_by_book: HashMap::new(),
        }
    }

    pub fn insert(&mut self, m: &Metadata, refs: References) {
        println!("-------------------- {} '{}'", &m.url, &m.header.title);

        for (book, cvs) in refs {
            println!("{} {}", book, cvs);
            // let post_refs = PostReferences::new(m, cvs);
            // let book_seen = self.refs_by_book.contains_key(book);
            // if book_seen {
            //     let mut existing_posts_refs = &mut self.refs_by_book[book];

            //     match existing_posts_refs.binary_search(&post_refs) {
            //         Ok(_i) => {
            //             // repeated insert, ignore
            //             println!("WARNING: repeated post insertion for {}", &m.url);
            //         }
            //         Err(i) => {
            //             existing_posts_refs.insert(i, post_refs);
            //         }
            //     }
            // } else {
            //     self.refs_by_book.insert(book, vec![post_refs]);
            // }
        }

        //for (k, v) in refs.into_iter() {}
    }
}
