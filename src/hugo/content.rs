use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    ContentRootNotFound,
    MissingPostHeader,
    NonUnicodePath,
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
            Self::NonUnicodePath => write!(f, "non-unicode path"),
        }
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Header {
    pub title: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
}

impl Header {
    pub fn new(title: &str, description: &str) -> Header {
        Header {
            title: Some(title.to_owned()),
            description: Some(description.to_owned()),
            date: None,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Metadata {
    pub path: PathBuf,
    pub url: String,
    pub header: Header,
}

impl Metadata {
    fn new(path: PathBuf, url: String, header: Header) -> Self {
        Metadata { path, url, header }
    }

    pub fn format_href(&self, sequence_number: &Option<usize>) -> String {
        let title = self.header.title.as_deref().unwrap_or("Unknown");
        match sequence_number {
            Some(sequence_number) => format_href(
                format!(
                    r###"{}<span style="font-size:smaller; padding-left:0.5em;">#{}</span>"###,
                    title, sequence_number
                )
                .as_str(),
                &self.url,
            ),
            None => format_href(title, &self.url),
        }
    }
}

pub fn format_href(text: &str, url: &str) -> String {
    format!("[{}]({{{{<relref \"{}\" >}}}})", text, url)
}

const CONTENT_DIR: &str = "content";

/// where Hugo posts live
pub const POSTS_SECTION: &str = "post";

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

    /// an iterator over the extraction of each page in the section
    pub fn section<T, F>(&self, section: &str, extractor: F) -> IntoIter<T, F>
    where
        F: Fn(&str, &str) -> T,
    {
        IntoIter {
            root: self.root.clone(),
            it: WalkDir::new(self.root.join(section))
                .sort_by_file_name()
                .into_iter(),
            extractor,
        }
    }

    pub fn section_writer(&self, section: &'static str) -> anyhow::Result<ContentWriter> {
        ContentWriter::new(&self.root, section)
    }
}

pub struct IntoIter<T, F>
where
    F: Fn(&str, &str) -> T,
{
    root: PathBuf,
    it: walkdir::IntoIter,
    extractor: F,
}

impl<T, F> IntoIter<T, F>
where
    F: Fn(&str, &str) -> T,
{
    fn parse(&self, f: &mut File, path: &Path, relpath: &Path) -> Result<(Metadata, T)> {
        match relpath.to_str() {
            None => Err(Error::NonUnicodePath.into()),
            Some(relpath) => {
                let mut content = String::new();
                f.read_to_string(&mut content).context(relpath.to_owned())?;

                let (header, raw_header, body) =
                    header_and_body(&content).context(relpath.to_owned())?;
                let metadata = Metadata::new(path.to_path_buf(), format!("/{}", relpath), header);

                let extracted = (self.extractor)(raw_header, body);

                Ok((metadata, extracted))
            }
        }
    }
}

impl<T, F> Iterator for IntoIter<T, F>
where
    F: Fn(&str, &str) -> T,
{
    type Item = Result<(Metadata, T)>;

    fn next(&mut self) -> Option<Result<(Metadata, T)>> {
        loop {
            match self.it.next() {
                None => break None,
                Some(Err(err)) => break Some(Err(err.into())),
                Some(Ok(entry)) => {
                    if entry.file_type().is_dir() {
                        let index_path = entry.path().join("index.md");
                        match File::open(&index_path) {
                            Ok(ref mut f) => {
                                // page bundle, so stop here
                                self.it.skip_current_dir();
                                let index_relpath = index_path.strip_prefix(&self.root).unwrap();
                                break Some(self.parse(f, &index_path, index_relpath));
                            }
                            Err(_) => continue,
                        }
                    } else {
                        let entry_path = entry.path();
                        let entry_relpath = entry_path.strip_prefix(&self.root).unwrap();

                        match File::open(entry.path()) {
                            Ok(ref mut f) => {
                                let result = self.parse(f, entry_path, entry_relpath);
                                break Some(result);
                            }
                            Err(e) => {
                                break Some(Err(anyhow::Error::from(e)
                                    .context(format!("{:?}", entry_relpath.display()))));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn header_and_body(text: &str) -> Result<(Header, &str, &str)> {
    lazy_static! {
        static ref HEADER_RE: Regex = Regex::new(r"(?s)\+\+\+(.*)(\+\+\+)").unwrap();
    }

    match HEADER_RE.captures(text) {
        Some(cap) => {
            let body = &text[cap.get(2).unwrap().end()..];
            let raw_header = &text[..cap.get(2).unwrap().end()];
            match toml::from_str::<Header>(&cap[1]) {
                Ok(header) => Ok((header, raw_header, body)),
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
