// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{BookChaptersVerses, ChaptersVerses};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Header {
    pub title: String,
}

pub struct Post {
    title: String,
    url: String,
}

impl Post {
    fn new(title: String, url: String) -> Self {
        Post { title, url }
    }
}

pub struct PostReferences<'a> {
    post: &'a Post,
    cvs: ChaptersVerses,
}

pub struct AllPostsReferences<'a> {
    all_posts: Vec<Post>,
    post_references_by_book: HashMap<&'static str, PostReferences<'a>>,
}

impl<'a> AllPostsReferences<'a> {
    pub fn new() -> Self {
        AllPostsReferences {
            all_posts: Vec::new(),
            post_references_by_book: HashMap::new(),
        }
    }

    fn add_post(&mut self, entry_relpath: &Path, header: Header) -> &Post {
        // this will panic on non-unicode paths, don't care for now
        self.all_posts.push(Post::new(
            header.title,
            entry_relpath.to_str().unwrap().to_string(),
        ));
        self.all_posts.last().unwrap()
    }

    pub fn add_post_refs(
        &mut self,
        entry_relpath: &Path,
        header: Header,
        _refs: BookChaptersVerses,
    ) {
        let _post = self.add_post(entry_relpath, header);

        //for (k, v) in refs.into_iter() {}
    }
}
