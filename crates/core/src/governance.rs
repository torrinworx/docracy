use crate::errors::GovernanceError;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GovernanceBundle {
    pub files: Vec<GovernanceFile>,
}

pub trait GovernanceSource {
    fn load_bundle(&self) -> Result<GovernanceBundle, GovernanceError>;
}

#[derive(Debug, Clone)]
pub struct FsGovernanceSource {
    dir: PathBuf,
}

impl FsGovernanceSource {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn repo_owned() -> Self {
        Self::new("./governance")
    }

    fn dir(&self) -> &Path {
        &self.dir
    }
}

impl GovernanceSource for FsGovernanceSource {
    fn load_bundle(&self) -> Result<GovernanceBundle, GovernanceError> {
        let mut files = Vec::new();
        let rd = fs::read_dir(self.dir()).map_err(|e| GovernanceError::Io(e.to_string()))?;
        for entry in rd {
            let entry = entry.map_err(|e| GovernanceError::Io(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.md")
                .to_string();
            let content =
                fs::read_to_string(&path).map_err(|e| GovernanceError::Io(e.to_string()))?;
            files.push(GovernanceFile { name, content });
        }

        // Stable ordering makes Init deterministic.
        files.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(GovernanceBundle { files })
    }
}
