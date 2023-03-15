#![feature(fn_traits)]
#![feature(unboxed_closures)]

use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use bible::References;
use clap::Parser;
use hugo::Metadata;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

struct PostHandler1<'a, 'b> {
    posts: &'a mut posts::Posts<'b>,
}

impl<'a, 'b> PostHandler1<'a, 'b> {
    fn new(posts: &'a mut posts::Posts<'b>) -> PostHandler1<'a, 'b> {
        PostHandler1 { posts }
    }
}

impl<'a, 'b> FnOnce<(Metadata, bible::References)> for PostHandler1<'a, 'b>
//where
//    'a: 'b,
{
    type Output = Result<()>;

    extern "rust-call" fn call_once(self, args: (Metadata, bible::References)) -> Self::Output {
        //self.posts.do_something(1, 2);
        //self.posts.insert(args.0, args.1);
        Ok(())
    }
}

struct PostHandler2<'a> {
    posts: &'a mut posts::Posts<'a>,
}

impl<'a> PostHandler2<'a> {
    fn new(posts: &'a mut posts::Posts<'a>) -> PostHandler2<'a> {
        PostHandler2 { posts }
    }
}

impl<'a> FnOnce<(Metadata, bible::References)> for PostHandler2<'a> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: (Metadata, bible::References)) -> Self::Output {
        self.posts.do_something(1, 2);
        //self.posts.insert(args.0, args.1);
    }
}

impl<'a> FnMut<(Metadata, bible::References)> for PostHandler2<'a> {
    extern "rust-call" fn call_mut(&mut self, args: (Metadata, bible::References)) {
        self.posts.do_something(1, 2);
        //self.posts.insert(args.0, args.1);
    }
}

fn main() -> ExitCode {
    let mut posts = posts::Posts::new();

    if let Err(e) = hugo::walk_posts(|metadata, body| {
        let refs = bible::get_references(body);

        //posts.insert(metadata, refs);

        // posts.insert(
        //     hugo::Metadata {
        //         url: "url".to_string(),
        //         header: hugo::Header {
        //             title: "My Title".to_string(),
        //         },
        //     },
        //     bible::References::new(),
        // );

        //posts.do_something("hello", &metadata, body);

        Ok::<(), anyhow::Error>(())
    }) {
        println!("failed: {:?}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

mod bible;
mod hugo;
mod posts;
mod util;

// use nightly toolchain
// Local Variables:
// rustic-cargo-bin: "cargo-nightly"
// End:
