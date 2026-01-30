pub mod cli;
pub mod config;
pub mod installer;
pub mod resolver;

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Config errors
    #[error("manifest not found at {0}")]
    ManifestNotFound(PathBuf),

    #[error("failed to parse manifest: {0}")]
    ManifestParse(String),

    #[error("failed to parse lock file: {0}")]
    LockFileParse(String),

    #[error("marketplace '{0}' not declared in manifest")]
    UndeclaredMarketplace(String),

    #[error("manifest already exists at {0}")]
    ManifestExists(PathBuf),

    // Resolver errors
    #[error("failed to clone marketplace '{name}': {source}")]
    MarketplaceClone {
        name: String,
        #[source]
        source: git2::Error,
    },

    #[error("failed to fetch marketplace '{name}': {source}")]
    MarketplaceFetch {
        name: String,
        #[source]
        source: git2::Error,
    },

    #[error("tag '{tag}' not found in marketplace '{marketplace}'")]
    TagNotFound { marketplace: String, tag: String },

    #[error("commit '{commit}' not found in marketplace '{marketplace}'")]
    CommitNotFound { marketplace: String, commit: String },

    #[error("marketplace.json not found in '{0}'")]
    MarketplaceJsonNotFound(String),

    #[error("failed to parse marketplace.json in '{name}': {reason}")]
    MarketplaceJsonParse { name: String, reason: String },

    #[error("plugin '{plugin}' not found in marketplace '{marketplace}'")]
    PluginNotFound { plugin: String, marketplace: String },

    // Installer errors
    #[error("failed to create cache directory: {0}")]
    CacheCreate(#[source] std::io::Error),

    #[error("failed to extract plugin '{0}': {1}")]
    PluginExtract(String, #[source] std::io::Error),

    #[error("failed to read {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse JSON in {path}: {source}")]
    JsonParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    // CLI errors
    #[error("plugin '{0}' not found in manifest")]
    PluginNotInManifest(String),

    #[error("no manifest found (run 'skill-manager init' first)")]
    NoManifest,

    #[error("operation aborted by user")]
    Aborted,

    // Git errors
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
