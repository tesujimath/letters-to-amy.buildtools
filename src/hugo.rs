use anyhow::Result;
use std::fs::File;

pub trait Create {
    fn create_branch(&mut self) -> Result<File>;

    // TODO return URL type not String
    fn create_leaf(&mut self, header: &Header) -> Result<(File, String)>;
}

mod content;
pub use content::{format_href, write_table, Content, Header, Metadata};

mod docs;
pub use docs::Docs;

mod home_links;
pub use home_links::contextualize as contextualize_home_links;
