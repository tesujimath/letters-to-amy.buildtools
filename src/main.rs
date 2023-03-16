use std::fs::{self, File};
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

    if let Some(root) = hugo::content_root() {
        let page_dir = root.join("page").join("index");
        let mut page_header = fs::read_to_string(page_dir.join("page-header.yaml")).unwrap();
        let mut outfile = File::create(page_dir.join("index.md")).unwrap();
        let index_dir = root.join("index");

        posts.dump(&page_header, outfile, &index_dir);
    }

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod posts;
mod util;
