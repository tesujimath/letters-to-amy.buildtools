use std::{
    io,
    path::{Path, PathBuf},
};

pub struct Docs {
    root: PathBuf,
}

pub type PageNumber = u8;

impl Docs {
    pub fn new() -> Result<Self, &'static str> {
        let candidate_roots = [Path::new("docs").to_owned(), Path::new("..").join("docs")];

        match candidate_roots.iter().find(|path| path.exists()) {
            Some(root) => Ok(Docs {
                root: root.to_path_buf(),
            }),
            None => Err("docs root not found, run from Hugo root or subdirectory of root"),
        }
    }

    /// return all pages except page 1 (which is simply home)
    pub fn pages(&self) -> io::Result<impl Iterator<Item = io::Result<(PageNumber, PathBuf)>>> {
        Ok(self.root.join("page").read_dir()?.filter_map(|r| match r {
            Ok(r) => r.file_name().to_str().and_then(|file_name| {
                file_name.parse::<PageNumber>().ok().and_then(|page| {
                    if page > 1 {
                        Some(Ok((page, r.path().join("index.html"))))
                    } else {
                        None
                    }
                })
            }),
            Err(e) => Some(Err(e)),
        }))
    }

    pub fn index_path(&self, href: &str) -> PathBuf {
        let rel_href = href.strip_prefix('/').unwrap_or(href);
        self.root.join(rel_href).join("index.html")
    }
}
