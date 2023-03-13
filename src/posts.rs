// TODO remove suppression for dead code warning
#![allow(dead_code)] //, unused_variables)]

use super::bible::{self, ChaptersVerses, References};
use std::collections::HashMap;

pub struct Post {
    url: String,
    title: String,
}

impl Post {
    fn new(url: &str, title: &str) -> Self {
        Post {
            url: url.to_owned(),
            title: title.to_owned(),
        }
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

    fn add_post(&mut self, url: &str, title: &str) -> &Post {
        self.all_posts.push(Post::new(url, title));
        self.all_posts.last().unwrap()
    }

    pub fn insert(&mut self, url: &str, title: &str, refs: &References) {
        println!("-------------------- {} '{}'", url, title);

        let _post = self.add_post(url, title);

        for book in bible::books() {
            if let Some(cvs) = refs.get(book) {
                println!("{} {}", book, cvs);
            }
        }

        //for (k, v) in refs.into_iter() {}
    }
}
