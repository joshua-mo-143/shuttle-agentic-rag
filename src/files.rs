use anyhow::Result;
use std::path::PathBuf;

pub struct File {
    pub path: String,
    pub contents: String,
    pub rows: Vec<String>,
}

impl File {
    pub fn new(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)?;

        let path_as_str = format!("{}", path.display());
        Ok(Self {
            path: path_as_str,
            contents,
            rows: Vec::new(),
        })
    }

    pub fn parse(mut self) -> Self {
        self.rows = self
            .contents
            .lines()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();

        self
    }
}
