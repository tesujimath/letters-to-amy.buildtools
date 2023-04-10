// TODO is this required? - mitigate recursion error when running tests
#![recursion_limit = "1024"]

use bible::{AllReferences, Writer};
use clap::{Parser, Subcommand};
use std::{io, path::PathBuf, process::ExitCode};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CreateScriptureIndex,
    ContextualizeHomeLinks,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateScriptureIndex => create_scripture_index(),
        Commands::ContextualizeHomeLinks => contextualize_home_links(),
    }
}

fn create_scripture_index() -> ExitCode {
    let content = hugo::content::Content::new().unwrap();
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

    refs.coelesce();
    refs.dump_repeats(io::stdout()).unwrap();

    const REF_SECTION: &str = "ref";

    let cw = content.section_writer(REF_SECTION).unwrap();
    let mut sw = Writer::new(cw);
    sw.write_references(&refs).unwrap();

    ExitCode::SUCCESS
}

fn contextualize_home_links() -> ExitCode {
    let docs = hugo::docs::Docs::new().unwrap();

    hugo::home_links::contextualize(&docs).unwrap();

    ExitCode::SUCCESS
}

mod bible;
mod hugo;
mod util;
