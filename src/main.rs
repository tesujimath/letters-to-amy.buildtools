// TODO is this required? - mitigate recursion error when running tests
#![recursion_limit = "1024"]

use clap::Parser;
use scripture_index::ScriptureIndexWriter;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    let content = hugo::Content::new().unwrap();
    let mut posts = posts::Posts::new();

    if let Err(e) = content.walk_posts(|metadata, body| {
        let (refs, warnings) = bible::references(body);

        for w in warnings {
            println!("WARN: {}: {}", &metadata.url, &w);
        }

        posts.insert(metadata, refs);
    }) {
        println!("failed: {:?}", e);
        return ExitCode::FAILURE;
    }

    const REF_SECTION: &str = "ref";

    let cw = content.section_writer(REF_SECTION).unwrap();
    let mut sw = ScriptureIndexWriter::new(cw);
    sw.write_posts(&posts).unwrap();

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod posts;
mod scripture_index;
mod util;
