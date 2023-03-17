use super::posts::{PostReferences, Posts};
use anyhow::{Context, Result};
use itertools::Itertools;
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
    pub title: String,
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

    pub fn scripture_index_writer(&self) -> anyhow::Result<ScriptureIndexWriter> {
        ScriptureIndexWriter::new(&self.root)
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

pub struct ScriptureIndexWriter {
    page_header: String,
    section_dir: PathBuf,
    index_markdown_file: File,
}

impl ScriptureIndexWriter {
    const SECTION_NAME: &str = "ref";

    fn new(content_root: &Path) -> anyhow::Result<Self> {
        let page_dir = content_root.join("page").join("scripture-index");
        let page_header_path = page_dir.join("page-header.yaml");
        let page_header = fs::read_to_string(&page_header_path)?;
        let index_markdown_path = page_dir.join("index.md");
        let index_markdown_file = File::create(index_markdown_path)?;
        let section_dir = content_root.join(Self::SECTION_NAME);

        fs::create_dir_all(&section_dir).unwrap();

        Ok(ScriptureIndexWriter {
            page_header,
            section_dir,
            index_markdown_file,
        })
    }

    const AUTOGEN_WARNING_YAML: &str = "# THIS FILE IS AUTO-GENERATED, DO NOT EDIT\n";
    const BOOK_REFS_DESCRIPTION: &str = "Scripture index";

    fn write_book_refs(
        &mut self,
        book: &str,
        refs: &Vec<PostReferences>,
        posts: &Posts,
    ) -> anyhow::Result<String> {
        let slug = slug::slugify(book);
        let path = self.section_dir.join(format!("{}.md", slug));

        let mut f = File::create(path).unwrap();
        f.write_all(
            format!(
                "---\n{}title: \"{}\"\ndescription: \"{}\"\n---\n\n| | |\n| --- | --- |\n",
                Self::AUTOGEN_WARNING_YAML,
                book,
                Self::BOOK_REFS_DESCRIPTION
            )
            .as_bytes(),
        )?;

        for r in refs {
            let m = &posts.metadata[r.post_index];
            f.write_all(
                format!(
                    "| [{}]({{{{<ref \"{}\" >}}}}) | {} |\n",
                    &m.header.title, &m.url, r
                )
                .as_bytes(),
            )?;
        }

        let abbrev = super::bible::abbrev(book).unwrap_or(book);
        let href = format!(
            "[{}]({{{{<ref \"/{}/{}\" >}}}})",
            abbrev,
            Self::SECTION_NAME,
            slug
        );

        Ok(href)
    }

    fn write_refs(
        &mut self,
        book_name_iter: impl Iterator<Item = &'static str>,
        hrefs: &mut Vec<String>,
        posts: &Posts,
    ) -> anyhow::Result<()> {
        for book in book_name_iter {
            if let Some(refs) = posts.refs_by_book.get(book) {
                let href = self.write_book_refs(book, refs, posts)?;
                hrefs.push(href);
            }
        }
        Ok(())
    }

    fn write_table(&mut self, heading: &str, hrefs: &Vec<String>) -> anyhow::Result<()> {
        self.index_markdown_file
            .write_all(format!("\n**{}**\n", heading).as_bytes())?;

        const ROW_SIZE: usize = 4;
        for _ in 0..ROW_SIZE {
            self.index_markdown_file.write_all("| ".as_bytes())?;
        }
        self.index_markdown_file.write_all("|\n".as_bytes())?;
        for _ in 0..ROW_SIZE {
            self.index_markdown_file.write_all("| --- ".as_bytes())?;
        }
        self.index_markdown_file.write_all("|\n".as_bytes())?;

        for href_batch in &hrefs.into_iter().chunks(ROW_SIZE) {
            for href in href_batch {
                self.index_markdown_file
                    .write_all(format!("| {} ", href).as_bytes())?;
            }
            self.index_markdown_file.write_all("|\n".as_bytes())?;
        }

        Ok(())
    }

    pub fn write_posts(&mut self, posts: &super::posts::Posts) -> anyhow::Result<()> {
        self.index_markdown_file.write_all(
            format!(
                "---\n{}{}---\n",
                Self::AUTOGEN_WARNING_YAML,
                self.page_header
            )
            .as_bytes(),
        )?;

        let mut ot_hrefs = Vec::new();
        let mut nt_hrefs = Vec::new();

        self.write_refs(super::bible::ot_books(), &mut ot_hrefs, posts)?;
        self.write_table("Old Testament", &ot_hrefs)?;

        self.write_refs(super::bible::nt_books(), &mut nt_hrefs, posts)?;
        self.write_table("New Testament", &nt_hrefs)?;

        Ok(())
    }
}

mod tests;
