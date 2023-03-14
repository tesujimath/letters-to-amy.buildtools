// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{ChaptersVerses, References};
use super::hugo::Metadata;
use std::collections::hash_map;
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

pub struct Posts<'a> {
    metadata: Vec<Metadata>,
    last_metadata: Option<Metadata>,
    refs: Vec<PostReferences<'a>>,
    last_post: Option<PostReferences<'a>>,
    refs_by_book: HashMap<&'static str, Vec<PostReferences<'a>>>,
}

impl<'a> Posts<'a> {
    pub fn new() -> Self {
        Posts {
            metadata: Vec::new(),
            last_metadata: None,
            refs: Vec::new(),
            last_post: None,
            refs_by_book: HashMap::new(),
        }
    }

    // the lifetime 'a here is what causes the problem with the borrowed data escaping:
    // or similarly the bound of 'a on 'b
    pub fn do_something(&'a mut self, a: &str, metadata: &Metadata, body: &str)
    //where
    //    'b: 'a,
    {
        println!("doing something with {} {:?} {}", a, metadata, body);
    }

    pub fn insert(&'a mut self, metadata: Metadata, refs: References) {
        println!(
            "-------------------- {} '{}'",
            &metadata.url, &metadata.header.title
        );

        //self.last_metadata = Some(metadata);
        //let m = self.last_metadata.as_ref().unwrap();

        self.metadata.push(metadata);
        let m = self.metadata.last().unwrap();

        for (book, cvs) in refs {
            println!("{} {}", book, &cvs);
            let post_refs = PostReferences::new(m, cvs);

            self.last_post = Some(post_refs); // None works here
                                              //self.refs.push(post_refs);

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
