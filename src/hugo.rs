use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::{
    fmt,
    fs::{self, DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

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
            &self.header.title.as_ref().unwrap_or(&"Unknown".to_string()),
            &self.url,
        )
    }
}

pub fn format_href(text: &str, url: &str) -> String {
    format!("[{}]({{{{<ref \"{}\" >}}}})", text, url)
}

pub struct Content {
    root: PathBuf,
}

impl Content {
    pub fn new() -> Result<Self, &'static str> {
        let candidate_roots = [
            Path::new("content").to_owned(),
            Path::new("..").join("content"),
        ];

        match candidate_roots.iter().find(|path| path.exists()) {
            Some(root) => Ok(Content {
                root: root.to_path_buf(),
            }),
            None => Err("content root not found, run from Hugo root or subdirectory of root"),
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
            .context("reading Hugo content to string")?;

        match (
            relpath.to_str(),
            header_and_body(&content).context("failed to get title and body"),
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
                    .context(format!("read_dir(\"{}\")", dir.to_string_lossy()))?)
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
                            .context(format!("open(\"{}\")", entry.path().to_string_lossy()))?;

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

#[derive(PartialEq, Eq, Debug)]
enum GetHeaderAndBodyErr {
    NoHeader,
    TomlError(toml::de::Error),
}

impl std::error::Error for GetHeaderAndBodyErr {}

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

fn header_and_body(text: &str) -> Result<(Header, &str), GetHeaderAndBodyErr> {
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

    pub fn create_branch(&mut self) -> anyhow::Result<File> {
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
    pub fn create_leaf(&mut self, header: &Header) -> anyhow::Result<(File, String)> {
        let unknown_title = "Unknown".to_string();
        let unknown_description = "".to_string();
        let title = header.title.as_ref().unwrap_or(&unknown_title);
        let description = header.description.as_ref().unwrap_or(&unknown_description);
        let slug = slug::slugify(title);
        let path = self.section_dir.join(format!("{}.md", slug));
        let url = format!("/{}/{}", self.section, slug);

        let mut f = File::create(path).unwrap();
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
