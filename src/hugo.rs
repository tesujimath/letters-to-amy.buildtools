use super::bible::extract_bible_refs;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::AsRef;
use std::fmt::Debug;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::{io, path::Path};

pub fn dump_all() -> io::Result<()> {
    walk_markdown_files("../content")
}

// TODO pass in callback
fn walk_markdown_files<P>(dir: P) -> io::Result<()>
where
    P: AsRef<Path> + Debug,
{
    fn is_markdown_file(e: &DirEntry) -> bool {
        let file_name = e.file_name();
        let p: &Path = file_name.as_ref();
        p.extension().and_then(|ext| ext.to_str()) == Some("md")
    }

    for entry_r in dir.as_ref().read_dir()? {
        if let Ok(entry) = entry_r {
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                walk_markdown_files(entry.path())?;
            } else if file_type.is_file() && is_markdown_file(&entry) {
                println!("Found markdown file {:?}", entry.path());
                let mut f = File::open(entry.path())?;

                let mut content = String::new();
                f.read_to_string(&mut content)?;

                extract_bible_refs(skip_header(&content));
            }
        }
    }

    Ok(())
}

fn skip_header(text: &str) -> &str {
    lazy_static! {
        static ref HEADER_RE: Regex = Regex::new(r"\+\+\+.*\+\+\+").unwrap();
    }

    match HEADER_RE.find(text) {
        Some(m) => &text[m.end()..],
        None => text,
    }
}
