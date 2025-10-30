use crate::config::{models::ConfigModel, Configuration};
use anyhow::bail;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

/// A configuration source
pub enum Source {
    /// A configuration file (maybe TOML or YAML)
    File(PathBuf),
}

const CANDIDATES: &[&str] = &[
    "Prank.toml",
    ".prank.toml",
    "Prank.yaml",
    ".prank.yaml",
    "Prank.json",
    ".prank.json",
];

impl Source {
    /// Find a first config source candidate in a directory
    pub fn find(path: &Path) -> anyhow::Result<Source> {
        for name in CANDIDATES {
            if let Some(file) = check_path(path, name) {
                return Ok(Source::File(file));
            }
        }
        bail!("Unable to find any Prank configuration");
    }

    /// Load the configuration from the source.
    ///
    /// This will validate and migrate anything that's required. It does not store any migrations.
    pub async fn load(self) -> anyhow::Result<Configuration> {
        match self {
            Self::File(file) => load_from(&file),
        }
        .and_then(|mut cfg| {
            cfg.migrate()?;
            Ok(cfg)
        })
    }
}

/// Load configuration from a file
///
/// Currently supported formats are:
///
/// * TOML
/// * YAML
/// * JSON
fn load_from(file: &Path) -> anyhow::Result<Configuration> {
    match file.extension().map(|s| s.to_string_lossy()).as_deref() {
        Some("toml") => Ok(toml::from_str(&String::from_utf8(std::fs::read(file)?)?)?),
        Some("yaml") => Ok(serde_yaml::from_reader(BufReader::new(File::open(file)?))?),
        Some("json") => Ok(serde_json::from_reader(BufReader::new(File::open(file)?))?),

        Some(n) => {
            bail!("Unsupported configuration file type: {n}");
        }
        None => {
            bail!("Missing configuration file extension");
        }
    }
}

/// Check if a file can be found in a directory.
fn check_path(path: &Path, name: &str) -> Option<PathBuf> {
    let path = path.join(name);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}
