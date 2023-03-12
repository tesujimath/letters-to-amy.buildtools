use super::bible;
use super::posts::Header;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::AsRef;
use std::fmt;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{io, path::Path};

pub fn dump_posts() -> io::Result<()> {
    let path = PathBuf::from("../content");
    get_all_post_bible_references(&path, &path, |p| p.starts_with("post/"))
}

// TODO pass in callback
fn get_all_post_bible_references<F>(root: &Path, dir: &Path, filter: F) -> io::Result<()>
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
                get_all_post_bible_references(root, &entry.path(), filter)?;
            } else if file_type.is_file() && is_markdown_file(&entry) {
                let entry_path = entry.path();
                let entry_relpath = entry_path.strip_prefix(root).unwrap();

                if filter(entry_relpath) {
                    let mut f = File::open(entry.path())?;

                    let mut content = String::new();
                    f.read_to_string(&mut content)?;

                    match get_header_and_body(&content) {
                        Ok((header, body)) => {
                            println!(
                                "==================== {} - '{}'",
                                entry_relpath.display(),
                                &header.title
                            );

                            let references = bible::get_references(body);

                            for book in bible::books() {
                                if let Some(cvs) = references.get(book) {
                                    println!("{} {}", book, cvs);
                                }
                            }
                        }
                        Err(e) => println!("failed to get title and body: {}", e),
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(PartialEq, Eq, Debug)]
enum GetHeaderAndBodyErr {
    NoHeader,
    TomlError(toml::de::Error),
}

impl fmt::Display for GetHeaderAndBodyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GetHeaderAndBodyErr::{}",
            match self {
                Self::NoHeader => "NoHeader".to_string(),
                Self::TomlError(e) => format!("TomlError: {}", e),
            }
        )
    }
}

fn get_header_and_body(text: &str) -> Result<(Header, &str), GetHeaderAndBodyErr> {
    lazy_static! {
        static ref HEADER_RE: Regex = Regex::new(r"(?s)\+\+\+(.*)(\+\+\+)").unwrap();
    }

    match HEADER_RE.captures(text) {
        Some(cap) => {
            let body = &text[cap.get(2).unwrap().end()..];
            match toml::from_str::<Header>(&cap[1]) {
                Ok(header) => Ok((header, body)),
                Err(e) => Err(GetHeaderAndBodyErr::TomlError(e)),
            }
        }
        None => Err(GetHeaderAndBodyErr::NoHeader),
    }
}

mod tests;
