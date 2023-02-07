use super::bible::extract_bible_refs;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::AsRef;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{io, path::Path};

pub fn dump_posts() -> io::Result<()> {
    let path = PathBuf::from("../content");
    walk_markdown_files(&path, &path, |p| p.starts_with("post/"))
}

// TODO pass in callback
fn walk_markdown_files<F>(root: &Path, dir: &Path, filter: F) -> io::Result<()>
where
    F: Fn(&Path) -> bool + Copy,
{
    fn is_markdown_file(e: &DirEntry) -> bool {
        let file_name = e.file_name();
        let p: &Path = file_name.as_ref();
        p.extension().and_then(|ext| ext.to_str()) == Some("md")
    }

    for entry_r in dir.read_dir()? {
        if let Ok(entry) = entry_r {
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                walk_markdown_files(root, &entry.path(), filter)?;
            } else if file_type.is_file() && is_markdown_file(&entry) {
                let entry_path = entry.path();
                let entry_relpath = entry_path.strip_prefix(root).unwrap();

                if filter(entry_relpath) {
                    println!("{}", entry_relpath.display());
                    let mut f = File::open(entry.path())?;

                    let mut content = String::new();
                    f.read_to_string(&mut content)?;

                    extract_bible_refs(skip_header(&content));
                }
            }
        }
    }

    Ok(())
}

fn skip_header(text: &str) -> &str {
    lazy_static! {
        static ref HEADER_RE: Regex = Regex::new(r"\+\+\+(?s:.*)\+\+\+").unwrap();
    }

    match HEADER_RE.find(text) {
        Some(m) => &text[m.end()..],
        None => text,
    }
}

mod tests;
