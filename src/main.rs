use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    let mut posts = posts::Posts::new();

    if let Err(e) = hugo::walk_posts(|metadata, body| {
        posts.insert(metadata, bible::get_references(body));
    }) {
        println!("failed: {:?}", e);
        return ExitCode::FAILURE;
    }

    posts.dump();

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod posts;
mod util;
