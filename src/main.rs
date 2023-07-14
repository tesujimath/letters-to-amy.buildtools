use anyhow::Result;
use bible::AllReferences;
use clap::{Parser, Subcommand};
use std::{
    fs::File,
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
    CreateScriptureIndexLinks,
    ContextualizeHomeLinks,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::CreateScriptureIndex => create_scripture_index(),
        Commands::CreateScriptureIndexLinks => create_scripture_index_links(),
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
    let content = hugo::Content::new()?;
    let mut refs = AllReferences::new();

    for r in content.section(hugo::POSTS_SECTION, bible::references) {
        match r {
            Ok((post_metadata, (post_refs, warnings))) => {
                let annotated_warnings = warnings
                    .into_iter()
                    .map(|w| format!("{}: {}", &post_metadata.url, w));
                for w in annotated_warnings {
                    println!("WARN: {}", &w);
                }

                refs.insert(post_metadata, post_refs);
            }
            Err(e) => println!("{:#}", e),
        }
    }

    const REF_SECTION: &str = "ref";
    let cw = content.section_writer(REF_SECTION)?;

    refs.tabulate(Box::new(cw))?;

    Ok(())
}

fn create_scripture_index_links() -> Result<()> {
    let content = hugo::Content::new()?;

    for r in content.section(hugo::POSTS_SECTION, bible::with_index_links) {
        match r {
            Ok((post_metadata, post_content)) => {
                if let Some(post_content) = post_content {
                    let mut f = File::create(&post_metadata.path)?;
                    println!("updating {}", post_metadata.path.to_str().unwrap());
                    f.write_all(post_content.as_bytes())?;
                }
            }
            Err(e) => println!("{:#}", e),
        }
    }

    Ok(())
}

fn contextualize_home_links() -> Result<()> {
    let docs = hugo::Docs::new()?;

    hugo::contextualize_home_links(&docs)?;

    Ok(())
}

mod bible;
mod hugo;
mod util;
