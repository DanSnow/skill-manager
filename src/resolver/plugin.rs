use git2::Repository;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::marketplace::{MarketplacePlugin, MarketplaceResolver, PluginSource};
use crate::config::SourceType;
use crate::layout::PluginLayout;
use crate::{Error, Result};

/// Metadata from plugin.json.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Resolved plugin information.
#[derive(Debug, Clone)]
pub struct ResolvedPlugin {
    pub name: String,
    pub marketplace: String,
    pub source_type: SourceType,
    pub marketplace_commit: String,
    pub plugin_commit: String,
    pub resolved_version: String,
    /// For local plugins: path within marketplace.
    /// For external plugins: URL of the plugin repo.
    pub source: String,
    /// Plugin directory layout for accessing plugin files.
    pub layout: PluginLayout,
}

impl ResolvedPlugin {
    /// Construct from a local plugin (within marketplace repo).
    ///
    /// Version is read from `.claude-plugin/plugin.json`. Falls back to first 7 chars
    /// of commit SHA if version is unavailable.
    pub fn from_local(
        name: String,
        marketplace: String,
        commit: String,
        source: String,
        layout: PluginLayout,
    ) -> Self {
        let resolved_version =
            Self::read_version(&layout).unwrap_or_else(|| commit[..7].to_string());

        Self {
            name,
            marketplace,
            source_type: SourceType::Local,
            marketplace_commit: commit.clone(),
            plugin_commit: commit,
            resolved_version,
            source,
            layout,
        }
    }

    /// Construct from an external plugin (separate git repo).
    ///
    /// Version is read from `.claude-plugin/plugin.json`. Falls back to first 7 chars
    /// of plugin_commit SHA if version is unavailable.
    pub fn from_external(
        name: String,
        marketplace: String,
        marketplace_commit: String,
        plugin_commit: String,
        source: String,
        layout: PluginLayout,
    ) -> Self {
        let resolved_version =
            Self::read_version(&layout).unwrap_or_else(|| plugin_commit[..7].to_string());

        Self {
            name,
            marketplace,
            source_type: SourceType::External,
            marketplace_commit,
            plugin_commit,
            resolved_version,
            source,
            layout,
        }
    }

    /// Read version from plugin.json, returns None if unavailable.
    fn read_version(layout: &PluginLayout) -> Option<String> {
        let content = std::fs::read_to_string(layout.plugin_json()).ok()?;
        let json: PluginJson = serde_json::from_str(&content).ok()?;
        json.version
    }
}

impl MarketplaceResolver {
    /// Resolve a plugin from a marketplace.
    pub fn resolve_plugin(
        &self,
        marketplace_name: &str,
        marketplace_commit: &str,
        plugin_name: &str,
        plugin_info: &MarketplacePlugin,
        requested_tag: Option<&str>,
        requested_commit: Option<&str>,
    ) -> Result<ResolvedPlugin> {
        match &plugin_info.source {
            PluginSource::Local(path) => {
                // Local plugin - lives within the marketplace repo
                self.resolve_local_plugin(
                    marketplace_name,
                    marketplace_commit,
                    plugin_name,
                    path,
                )
            }
            PluginSource::External { url, .. } => {
                // External plugin - separate git repository
                self.resolve_external_plugin(
                    marketplace_name,
                    marketplace_commit,
                    plugin_name,
                    url,
                    requested_tag,
                    requested_commit,
                )
            }
        }
    }

    /// Resolve a local plugin (path within marketplace).
    fn resolve_local_plugin(
        &self,
        marketplace_name: &str,
        marketplace_commit: &str,
        plugin_name: &str,
        path: &str,
    ) -> Result<ResolvedPlugin> {
        let marketplace_path = self.marketplace_path(marketplace_name);
        let plugin_path = marketplace_path.join(path);
        let layout = PluginLayout::new(&plugin_path);

        Ok(ResolvedPlugin::from_local(
            plugin_name.to_string(),
            marketplace_name.to_string(),
            marketplace_commit.to_string(),
            path.to_string(),
            layout,
        ))
    }

    /// Resolve an external plugin (separate git repository).
    fn resolve_external_plugin(
        &self,
        marketplace_name: &str,
        marketplace_commit: &str,
        plugin_name: &str,
        url: &str,
        requested_tag: Option<&str>,
        requested_commit: Option<&str>,
    ) -> Result<ResolvedPlugin> {
        // Clone/fetch the external plugin repo
        let plugin_cache_path = self.plugin_repo_path(marketplace_name, plugin_name);

        let repo = if plugin_cache_path.exists() {
            self.fetch_plugin_repo(plugin_name, &plugin_cache_path)?
        } else {
            self.clone_plugin_repo(plugin_name, url, &plugin_cache_path)?
        };

        // Resolve the version
        let plugin_commit = if let Some(commit) = requested_commit {
            commit.to_string()
        } else if let Some(tag) = requested_tag {
            self.resolve_tag(&repo, plugin_name, tag)?
        } else {
            self.resolve_head(&repo)?
        };

        // Checkout the resolved commit
        self.checkout_commit(&repo, plugin_name, &plugin_commit)?;

        // Read version from plugin.json using PluginLayout
        let workdir = repo.workdir().unwrap_or(&plugin_cache_path);
        let layout = PluginLayout::new(workdir);

        Ok(ResolvedPlugin::from_external(
            plugin_name.to_string(),
            marketplace_name.to_string(),
            marketplace_commit.to_string(),
            plugin_commit,
            url.to_string(),
            layout,
        ))
    }

    /// Get the cache path for an external plugin repo.
    fn plugin_repo_path(&self, marketplace: &str, plugin: &str) -> PathBuf {
        self.cache_dir
            .join("plugin-repos")
            .join(marketplace)
            .join(plugin)
    }

    /// Clone an external plugin repository.
    fn clone_plugin_repo(&self, name: &str, url: &str, path: &Path) -> Result<Repository> {
        std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
            .map_err(Error::CacheCreate)?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    return git2::Cred::ssh_key_from_agent(username);
                }
            }
            git2::Cred::default()
        });

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        builder.clone(url, path).map_err(|e| Error::MarketplaceClone {
            name: name.to_string(),
            source: e,
        })
    }

    /// Fetch updates for an external plugin repository.
    fn fetch_plugin_repo(&self, name: &str, path: &Path) -> Result<Repository> {
        let repo = Repository::open(path).map_err(|e| Error::MarketplaceClone {
            name: name.to_string(),
            source: e,
        })?;

        {
            let mut remote = repo.find_remote("origin").map_err(|e| Error::MarketplaceFetch {
                name: name.to_string(),
                source: e,
            })?;

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, allowed_types| {
                if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                    if let Some(username) = username_from_url {
                        return git2::Cred::ssh_key_from_agent(username);
                    }
                }
                git2::Cred::default()
            });

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);

            remote
                .fetch(
                    &["refs/heads/*:refs/heads/*", "refs/tags/*:refs/tags/*"],
                    Some(&mut fo),
                    None,
                )
                .map_err(|e| Error::MarketplaceFetch {
                    name: name.to_string(),
                    source: e,
                })?;
        }

        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_marketplace_with_local_plugin(dir: &Path, include_version: bool) -> Repository {
        let repo = Repository::init(dir).unwrap();

        // Create marketplace.json
        let marketplace_json = r#"{
            "plugins": {
                "local-plugin": {
                    "path": "plugins/local-plugin",
                    "description": "A local plugin"
                }
            }
        }"#;
        fs::write(dir.join("marketplace.json"), marketplace_json).unwrap();

        // Create local plugin with plugin.json in .claude-plugin/
        let plugin_dir = dir.join("plugins/local-plugin");
        let config_dir = plugin_dir.join(".claude-plugin");
        fs::create_dir_all(&config_dir).unwrap();

        if include_version {
            let plugin_json = r#"{
                "name": "local-plugin",
                "version": "1.2.3",
                "description": "Test local plugin"
            }"#;
            fs::write(config_dir.join("plugin.json"), plugin_json).unwrap();
        }

        // Commit everything
        {
            let mut index = repo.index().unwrap();
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }

        repo
    }

    #[test]
    fn test_resolve_local_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let marketplace_dir = temp_dir.path().join("marketplaces/test");
        fs::create_dir_all(&marketplace_dir).unwrap();

        // Include version in plugin.json
        let repo = setup_marketplace_with_local_plugin(&marketplace_dir, true);
        let commit = repo.head().unwrap().peel_to_commit().unwrap().id().to_string();

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let plugin_info = super::super::marketplace::MarketplacePlugin {
            name: "local-plugin".to_string(),
            source: PluginSource::Local("plugins/local-plugin".to_string()),
            description: Some("A local plugin".to_string()),
        };

        let resolved = resolver
            .resolve_plugin("test", &commit, "local-plugin", &plugin_info, None, None)
            .unwrap();

        assert_eq!(resolved.name, "local-plugin");
        assert_eq!(resolved.marketplace, "test");
        assert_eq!(resolved.source_type, SourceType::Local);
        assert_eq!(resolved.marketplace_commit, commit);
        assert_eq!(resolved.plugin_commit, commit);
        // Version should come from .claude-plugin/plugin.json
        assert_eq!(resolved.resolved_version, "1.2.3");
        assert_eq!(resolved.source, "plugins/local-plugin");
    }

    #[test]
    fn test_sha_fallback_when_plugin_json_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let marketplace_dir = temp_dir.path().join("marketplaces/test");
        fs::create_dir_all(&marketplace_dir).unwrap();

        // No plugin.json created (include_version = false)
        let repo = setup_marketplace_with_local_plugin(&marketplace_dir, false);
        let commit = repo.head().unwrap().peel_to_commit().unwrap().id().to_string();

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let plugin_info = super::super::marketplace::MarketplacePlugin {
            name: "local-plugin".to_string(),
            source: PluginSource::Local("plugins/local-plugin".to_string()),
            description: Some("A local plugin".to_string()),
        };

        let resolved = resolver
            .resolve_plugin("test", &commit, "local-plugin", &plugin_info, None, None)
            .unwrap();

        // Version should fallback to first 7 chars of commit SHA
        assert_eq!(resolved.resolved_version, &commit[..7]);
    }

    #[test]
    fn test_sha_fallback_when_version_field_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let marketplace_dir = temp_dir.path().join("marketplaces/test");
        fs::create_dir_all(&marketplace_dir).unwrap();

        // Create plugin.json without version field
        let repo = Repository::init(&marketplace_dir).unwrap();

        let marketplace_json = r#"{
            "plugins": {
                "local-plugin": {
                    "path": "plugins/local-plugin",
                    "description": "A local plugin"
                }
            }
        }"#;
        fs::write(marketplace_dir.join("marketplace.json"), marketplace_json).unwrap();

        let plugin_dir = marketplace_dir.join("plugins/local-plugin");
        let config_dir = plugin_dir.join(".claude-plugin");
        fs::create_dir_all(&config_dir).unwrap();

        // plugin.json exists but has no version field
        let plugin_json = r#"{"name": "local-plugin"}"#;
        fs::write(config_dir.join("plugin.json"), plugin_json).unwrap();

        // Commit everything
        {
            let mut index = repo.index().unwrap();
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }

        let commit = repo.head().unwrap().peel_to_commit().unwrap().id().to_string();

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let plugin_info = super::super::marketplace::MarketplacePlugin {
            name: "local-plugin".to_string(),
            source: PluginSource::Local("plugins/local-plugin".to_string()),
            description: Some("A local plugin".to_string()),
        };

        let resolved = resolver
            .resolve_plugin("test", &commit, "local-plugin", &plugin_info, None, None)
            .unwrap();

        // Version should fallback to first 7 chars of commit SHA
        assert_eq!(resolved.resolved_version, &commit[..7]);
    }
}
