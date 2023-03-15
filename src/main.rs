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
