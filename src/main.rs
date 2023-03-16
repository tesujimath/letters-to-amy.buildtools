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
        let (refs, warnings) = bible::references(body);

        for w in warnings {
            println!("WARN: {}: {}", &metadata.url, &w);
        }

        posts.insert(metadata, refs);
    }) {
        println!("failed: {:?}", e);
        return ExitCode::FAILURE;
    }

    if let Some(root) = hugo::content_root() {
        let page_dir = root.join("page").join("scripture-index");
        let mut page_header = fs::read_to_string(page_dir.join("page-header.yaml")).unwrap();
        let mut outfile = File::create(page_dir.join("index.md")).unwrap();
        let section_name = "ref";
        let section_dir = root.join(section_name);

        fs::create_dir_all(&section_dir).unwrap();
        posts.dump(&page_header, outfile, &section_dir, section_name);
    }

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod posts;
mod util;
