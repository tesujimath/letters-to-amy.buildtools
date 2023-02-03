use std::convert::AsRef;
use std::fmt::Debug;
use std::fs::DirEntry;
use std::{io, path::Path};

pub fn dump_all() -> io::Result<()> {
    dump_refs("../content")
}

fn dump_refs<P>(dir: P) -> io::Result<()>
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
                dump_refs(entry.path())?;
            } else if file_type.is_file() && is_markdown_file(&entry) {
                println!("Found markdown file {:?}", entry.path());
            }
        }
    }

    Ok(())
}
