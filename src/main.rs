use anyhow::Result;
use bible::AllReferences;
use clap::{Parser, Subcommand};
use std::{
    io::{stderr, Write},
    path::PathBuf,
    process::ExitCode,
};

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

    let result = match &cli.command {
        Commands::CreateScriptureIndex => create_scripture_index(),
        Commands::ContextualizeHomeLinks => contextualize_home_links(),
    };

    if let Err(e) = result {
        let _ = writeln!(stderr(), "error: {:#}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn create_scripture_index() -> Result<()> {
    let content = hugo::content::Content::new()?;
    let mut refs = AllReferences::new();

    content.walk_posts(|metadata, body| {
        let warnings = refs.extract_from_post(metadata, body);

        for w in warnings {
            println!("WARN: {}", &w);
        }
    })?;

    const REF_SECTION: &str = "ref";
    let cw = content.section_writer(REF_SECTION)?;

    refs.tabulate(cw)?;

    Ok(())
}

fn contextualize_home_links() -> Result<()> {
    let docs = hugo::docs::Docs::new()?;

    hugo::home_links::contextualize(&docs)?;

    Ok(())
}

mod bible;
mod hugo;
mod util;
