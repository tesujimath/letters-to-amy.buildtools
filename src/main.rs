use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    // TODO the rest
}

fn main() -> ExitCode {
    if let Err(e) = hugo::walk_posts(|url, header, body| {
        println!("-------------------- {} '{}'", url, header.title);

        let references = bible::get_references(body);

        for book in bible::books() {
            if let Some(cvs) = references.get(book) {
                println!("{} {}", book, cvs);
            }
        }

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
