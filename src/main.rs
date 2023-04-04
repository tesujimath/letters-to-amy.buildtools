// TODO is this required? - mitigate recursion error when running tests
#![recursion_limit = "1024"]

use bible::{AllReferences, Writer};
use clap::Parser;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    let content = hugo::Content::new().unwrap();
    let mut refs = AllReferences::new();

    if let Err(e) = content.walk_posts(|metadata, body| {
        let warnings = refs.extract_from_post(metadata, body);

        for w in warnings {
            println!("WARN: {}", &w);
        }
    }) {
        println!("failed: {:?}", e);
        return ExitCode::FAILURE;
    }

    const REF_SECTION: &str = "ref";

    let cw = content.section_writer(REF_SECTION).unwrap();
    let mut sw = Writer::new(cw);
    sw.write_references(&refs).unwrap();

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod util;
