use super::posts::Header;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::{io, path::Path};

pub fn walk_posts<F>(handler: F) -> io::Result<()>
where
    F: Fn(&str, &Header, &str) + Copy,
{
    let roots = [
        Path::new("content").to_owned(),
        Path::new("..").join("content"),
    ];

    match roots.iter().find(|path| path.exists()) {
        Some(root) => {
            let posts_path = root.join("post");
            walk(&posts_path, &posts_path, handler)
        }
        None => {
            panic!("ERROR: content root not found, run from Hugo root or subdirectory of root");
        }
    }
}

fn walk<F>(root: &Path, dir: &Path, handler: F) -> io::Result<()>
where
    F: Fn(&str, &Header, &str) + Copy,
{
    fn parse<F1>(f: &mut File, relpath: &Path, handler: F1) -> io::Result<()>
    where
        F1: Fn(&str, &Header, &str) + Copy,
    {
        let mut content = String::new();
        f.read_to_string(&mut content)?;

        match (relpath.to_str(), get_header_and_body(&content)) {
            (Some(relpath), Ok((header, body))) => {
                handler(relpath, &header, body);
            }
            (None, _) => {
                println!("WARNING: skipping non-unicode path {:?}", relpath);
            }
            (_, Err(e)) => {
                println!("failed to get title and body: {}", e);
            }
        }

        // TODO return error, work out how to stack up errors of different types - enum?
        Ok(())
    }

    let index_path = dir.join("index.md");
    match File::open(&index_path) {
        Ok(ref mut f) => {
            // page bundle, so stop here
            let index_relpath = index_path.strip_prefix(root).unwrap();
            parse(f, index_relpath, handler)?;
        }
        Err(_) => {
            // no page bundle, so walk further
            for entry in (dir.read_dir()?).flatten() {
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    walk(root, &entry.path(), handler)?;
                } else if file_type.is_file() {
                    let entry_path = entry.path();
                    let entry_relpath = entry_path.strip_prefix(root).unwrap();

                    let mut f = File::open(entry.path())?;

                    parse(&mut f, entry_relpath, handler)?;
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
