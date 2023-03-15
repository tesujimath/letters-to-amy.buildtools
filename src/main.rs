use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    let mut posts_list = Vec::new();

    if let Err(e) = hugo::walk_posts(|metadata, body| {
        posts_list.push((metadata, bible::get_references(body)));
    }) {
        println!("failed: {:?}", e);
        return ExitCode::FAILURE;
    }

    let mut posts = posts::Posts::new();
    for (m, refs) in &posts_list {
        posts.insert(m, refs);
    }

    posts.dump();

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod posts;
mod util;
