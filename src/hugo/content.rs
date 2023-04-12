use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    fs::{self, DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    ContentRootNotFound,
    MissingPostHeader,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::ContentRootNotFound => write!(
                f,
                "{} directory not found, run from repo root directory or a subdirectory",
                CONTENT_DIR
            ),
            Self::MissingPostHeader => write!(f, "missing header in post"),
        }
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Header {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Header {
    pub fn new(title: &str, description: &str) -> Header {
        Header {
            title: Some(title.to_owned()),
            description: Some(description.to_owned()),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Metadata {
    pub url: String,
    pub header: Header,
}

impl Metadata {
    fn new(url: String, header: Header) -> Self {
        Metadata { url, header }
    }

    pub fn format_href(&self) -> String {
        format_href(
            self.header.title.as_ref().unwrap_or(&"Unknown".to_string()),
            &self.url,
        )
    }
}

pub fn format_href(text: &str, url: &str) -> String {
    format!("[{}]({{{{<relref \"{}\" >}}}})", text, url)
}

const CONTENT_DIR: &str = "content";

#[derive(Debug)]
pub struct Content {
    root: PathBuf,
}

impl Content {
    pub fn new() -> Result<Self, Error> {
        let candidate_roots = [
            Path::new(CONTENT_DIR).to_owned(),
            Path::new("..").join(CONTENT_DIR),
        ];

        match candidate_roots.iter().find(|path| path.exists()) {
            Some(root) => Ok(Content {
                root: root.to_path_buf(),
            }),
            None => Err(Error::ContentRootNotFound),
        }
    }

    pub fn walk_posts<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Metadata, &str),
    {
        let posts_path = self.root.join("post");
        self.walk(&posts_path, &mut handler)
    }

    fn parse<F>(&self, f: &mut File, relpath: &Path, handler: &mut F) -> Result<()>
    where
        F: FnMut(Metadata, &str),
    {
        let mut content = String::new();
        f.read_to_string(&mut content)
            .context(format!("{}", relpath.to_string_lossy()))?;

        match (
            relpath.to_str(),
            header_and_body(&content).context(format!("{}", relpath.to_string_lossy())),
        ) {
            (Some(relpath), Ok((header, body))) => {
                let metadata = Metadata::new(format!("/{}", relpath), header);

                handler(metadata, body);

                Ok(())
            }
            (None, _) => {
                println!("WARNING: skipping non-unicode path {:?}", relpath);
                Ok(())
            }
            (_, Err(e)) => Err(e),
        }
    }

    fn walk<F>(&self, dir: &Path, handler: &mut F) -> Result<()>
    where
        F: FnMut(Metadata, &str),
    {
        let index_path = dir.join("index.md");
        match File::open(&index_path) {
            Ok(ref mut f) => {
                // page bundle, so stop here
                let index_relpath = index_path.strip_prefix(&self.root).unwrap();
                self.parse(f, index_relpath, handler)?
            }
            Err(_) => {
                // no page bundle, so walk further
                let mut entries = (dir
                    .read_dir()
                    .context(format!("read_dir(\"{}\")", dir.display()))?)
                .flatten()
                // sort by name, to provide a defined order of iteration
                .collect::<Vec<DirEntry>>();
                entries.sort_by_key(|e| e.file_name());
                for entry in entries {
                    let file_type = entry.file_type()?;
                    if file_type.is_dir() {
                        self.walk(&entry.path(), handler)?;
                    } else if file_type.is_file() {
                        let entry_path = entry.path();
                        let entry_relpath = entry_path.strip_prefix(&self.root).unwrap();

                        let mut f = File::open(entry.path())
                            .context(format!("open(\"{}\")", entry.path().display()))?;

                        self.parse(&mut f, entry_relpath, handler)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn section_writer(&self, section: &'static str) -> anyhow::Result<ContentWriter> {
        ContentWriter::new(&self.root, section)
    }
}

fn header_and_body(text: &str) -> Result<(Header, &str)> {
    lazy_static! {
        static ref HEADER_RE: Regex = Regex::new(r"(?s)\+\+\+(.*)(\+\+\+)").unwrap();
    }

    match HEADER_RE.captures(text) {
        Some(cap) => {
            let body = &text[cap.get(2).unwrap().end()..];
            match toml::from_str::<Header>(&cap[1]) {
                Ok(header) => Ok((header, body)),
                Err(e) => Err(e.into()),
            }
        }
        None => Err(Error::MissingPostHeader.into()),
    }
}

#[derive(Debug)]
pub struct ContentWriter {
    section: &'static str,
    section_dir: PathBuf,
    branch_yaml_header: String,
}

impl ContentWriter {
    const AUTOGEN_WARNING_YAML: &str = "# THIS FILE IS AUTO-GENERATED, DO NOT EDIT\n";

    fn new(content_root: &Path, section: &'static str) -> anyhow::Result<Self> {
        let section_dir = content_root.join(section);

        let archetypes_dir = content_root.join("..").join("archetypes");
        let branch_yaml_header_path = archetypes_dir.join(format!("{}.yaml", section));
        let branch_yaml_header = fs::read_to_string(branch_yaml_header_path)?;

        fs::create_dir_all(&section_dir)?;

        Ok(ContentWriter {
            section,
            section_dir,
            branch_yaml_header,
        })
    }
}

impl super::Create for ContentWriter {
    fn create_branch(&mut self) -> anyhow::Result<File> {
        let index_path = self.section_dir.join("_index.md");
        let mut f = File::create(index_path)?;

        f.write_all(
            format!(
                "---\n{}{}---\n",
                Self::AUTOGEN_WARNING_YAML,
                self.branch_yaml_header
            )
            .as_bytes(),
        )?;

        Ok(f)
    }

    // TODO return URL type not String
    fn create_leaf(&mut self, header: &Header) -> anyhow::Result<(File, String)> {
        let unknown_title = "Unknown".to_string();
        let unknown_description = "".to_string();
        let title = header.title.as_ref().unwrap_or(&unknown_title);
        let description = header.description.as_ref().unwrap_or(&unknown_description);
        let slug = slug::slugify(title);
        let path = self.section_dir.join(format!("{}.md", slug));
        let url = format!("/{}/{}", self.section, slug);

        let mut f = File::create(path)?;
        f.write_all(
            // TODO use YAML serializer
            format!(
                "---\n{}title: \"{}\"\ndescription: \"{}\"\n---\n",
                Self::AUTOGEN_WARNING_YAML,
                title,
                description
            )
            .as_bytes(),
        )?;

        Ok((f, url))
    }
}

/// write a table;  it is the callers responsibility to ensure that all rows are the same length, and match the header
pub fn write_table(
    mut f: impl Write,
    header: impl IntoIterator<Item = impl fmt::Display>,
    rows: impl IntoIterator<Item = impl IntoIterator<Item = impl fmt::Display>>,
) -> std::io::Result<()> {
    let mut width = 0;
    for field in header {
        width += 1;
        f.write_all(format!("| {} ", field).as_bytes())?;
    }
    f.write_all("|\n".as_bytes())?;
    for _ in 0..width {
        f.write_all("| --- ".as_bytes())?;
    }
    f.write_all("|\n".as_bytes())?;

    for row in rows {
        for field in row {
            f.write_all(format!("| {} ", field).as_bytes())?;
        }
        f.write_all("|\n".as_bytes())?;
    }

    Ok(())
}

mod tests;
