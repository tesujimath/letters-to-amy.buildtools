use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    let mut _all_references = posts::AllPostsReferences::new();

    if let Err(e) = hugo::walk_posts(|url, header, body| {
        let post_references = bible::get_references(body);

        _all_references.insert(url, &header.title, &post_references);

        Ok(())
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
