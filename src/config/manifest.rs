use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// A marketplace URL with optional version pinning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceEntry {
    pub url: String,
    pub tag: Option<String>,
    pub commit: Option<String>,
}

/// A plugin entry with marketplace reference and optional version pinning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEntry {
    pub marketplace: String,
    pub tag: Option<String>,
    pub commit: Option<String>,
}

/// The parsed plugins.toml manifest.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub marketplaces: HashMap<String, MarketplaceEntry>,
    pub plugins: HashMap<String, PluginEntry>,
    pub path: Option<PathBuf>,
}

// Internal structs for TOML deserialization
#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    marketplaces: HashMap<String, RawMarketplace>,
    #[serde(default)]
    plugins: HashMap<String, RawPlugin>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawMarketplace {
    Simple(String),
    Detailed(MarketplaceDetails),
}

#[derive(Debug, Deserialize)]
struct MarketplaceDetails {
    url: String,
    tag: Option<String>,
    commit: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawPlugin {
    marketplace: String,
    tag: Option<String>,
    commit: Option<String>,
}

/// Manifest file locations.
pub const MANIFEST_FILENAME: &str = "plugins.toml";

impl Manifest {
    /// Get the global manifest path (~/.config/skill-manager/plugins.toml).
    pub fn global_path() -> Option<PathBuf> {
        let dirs = xdg::BaseDirectories::with_prefix("skill-manager");
        dirs.get_config_home().map(|p| p.join(MANIFEST_FILENAME))
    }

    /// Get the project manifest path (./.claude/plugins.toml).
    pub fn project_path() -> PathBuf {
        PathBuf::from(".claude").join(MANIFEST_FILENAME)
    }

    /// Load the global manifest if it exists.
    pub fn load_global() -> Result<Option<Self>> {
        if let Some(path) = Self::global_path() {
            if path.exists() {
                return Ok(Some(Self::load(&path)?));
            }
        }
        Ok(None)
    }

    /// Load the project manifest if it exists.
    pub fn load_project() -> Result<Option<Self>> {
        let path = Self::project_path();
        if path.exists() {
            return Ok(Some(Self::load(&path)?));
        }
        Ok(None)
    }

    /// Parse a manifest from TOML content.
    pub fn parse(content: &str) -> Result<Self> {
        let raw: RawManifest =
            toml::from_str(content).map_err(|e| Error::ManifestParse(e.to_string()))?;

        let marketplaces = raw
            .marketplaces
            .into_iter()
            .map(|(name, raw)| {
                let entry = match raw {
                    RawMarketplace::Simple(url) => MarketplaceEntry {
                        url: expand_github_shorthand(&url),
                        tag: None,
                        commit: None,
                    },
                    RawMarketplace::Detailed(details) => MarketplaceEntry {
                        url: expand_github_shorthand(&details.url),
                        tag: details.tag,
                        commit: details.commit,
                    },
                };
                (name, entry)
            })
            .collect();

        let plugins = raw
            .plugins
            .into_iter()
            .map(|(name, raw)| {
                let entry = PluginEntry {
                    marketplace: raw.marketplace,
                    tag: raw.tag,
                    commit: raw.commit,
                };
                (name, entry)
            })
            .collect();

        Ok(Manifest {
            marketplaces,
            plugins,
            path: None,
        })
    }

    /// Load a manifest from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;
        let mut manifest = Self::parse(&content)?;
        manifest.path = Some(path.to_path_buf());
        Ok(manifest)
    }

    /// Validate that all plugins reference declared marketplaces.
    pub fn validate(&self) -> Result<()> {
        for (_plugin_name, plugin) in &self.plugins {
            if !self.marketplaces.contains_key(&plugin.marketplace) {
                return Err(Error::UndeclaredMarketplace(plugin.marketplace.clone()));
            }
        }
        Ok(())
    }
}

/// Expand GitHub shorthand (owner/repo) to full HTTPS URL.
/// SSH and HTTPS URLs are passed through unchanged.
fn expand_github_shorthand(url: &str) -> String {
    if url.starts_with("git@") || url.starts_with("https://") || url.starts_with("http://") {
        url.to_string()
    } else if url.contains('/') && !url.contains(':') {
        // Looks like owner/repo format
        format!("https://github.com/{}.git", url)
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_github_shorthand() {
        assert_eq!(
            expand_github_shorthand("anthropics/claude-plugins-official"),
            "https://github.com/anthropics/claude-plugins-official.git"
        );
        assert_eq!(
            expand_github_shorthand("git@github.com:mycompany/plugins.git"),
            "git@github.com:mycompany/plugins.git"
        );
        assert_eq!(
            expand_github_shorthand("https://git.example.com/plugins.git"),
            "https://git.example.com/plugins.git"
        );
    }

    #[test]
    fn test_parse_minimal_manifest() {
        let content = r#"
[marketplaces]

[plugins]
"#;
        let manifest = Manifest::parse(content).unwrap();
        assert!(manifest.marketplaces.is_empty());
        assert!(manifest.plugins.is_empty());
    }

    #[test]
    fn test_parse_simple_marketplace() {
        let content = r#"
[marketplaces]
official = "anthropics/claude-plugins-official"

[plugins]
"#;
        let manifest = Manifest::parse(content).unwrap();
        assert_eq!(
            manifest.marketplaces["official"].url,
            "https://github.com/anthropics/claude-plugins-official.git"
        );
    }

    #[test]
    fn test_parse_detailed_marketplace() {
        let content = r#"
[marketplaces]
pinned = { url = "owner/repo", tag = "v1.0" }

[plugins]
"#;
        let manifest = Manifest::parse(content).unwrap();
        let entry = &manifest.marketplaces["pinned"];
        assert_eq!(entry.url, "https://github.com/owner/repo.git");
        assert_eq!(entry.tag, Some("v1.0".to_string()));
    }

    #[test]
    fn test_parse_plugin() {
        let content = r#"
[marketplaces]
official = "anthropics/claude-plugins-official"

[plugins]
typescript-lsp = { marketplace = "official" }
superpowers = { marketplace = "official", tag = "v4.1.1" }
sourceatlas = { marketplace = "official", commit = "def456" }
"#;
        let manifest = Manifest::parse(content).unwrap();

        let ts = &manifest.plugins["typescript-lsp"];
        assert_eq!(ts.marketplace, "official");
        assert_eq!(ts.tag, None);
        assert_eq!(ts.commit, None);

        let sp = &manifest.plugins["superpowers"];
        assert_eq!(sp.tag, Some("v4.1.1".to_string()));

        let sa = &manifest.plugins["sourceatlas"];
        assert_eq!(sa.commit, Some("def456".to_string()));
    }

    #[test]
    fn test_validate_undeclared_marketplace() {
        let content = r#"
[marketplaces]

[plugins]
myplugin = { marketplace = "unknown" }
"#;
        let manifest = Manifest::parse(content).unwrap();
        let result = manifest.validate();
        assert!(matches!(result, Err(Error::UndeclaredMarketplace(_))));
    }
}
