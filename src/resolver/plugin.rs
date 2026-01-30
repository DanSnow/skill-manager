use git2::Repository;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::marketplace::{MarketplacePlugin, MarketplaceResolver};
use crate::config::SourceType;
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
        if let Some(path) = &plugin_info.path {
            // Local plugin - lives within the marketplace repo
            self.resolve_local_plugin(
                marketplace_name,
                marketplace_commit,
                plugin_name,
                path,
            )
        } else if let Some(url) = &plugin_info.url {
            // External plugin - separate git repository
            self.resolve_external_plugin(
                marketplace_name,
                marketplace_commit,
                plugin_name,
                url,
                requested_tag,
                requested_commit,
            )
        } else {
            Err(Error::PluginNotFound {
                plugin: plugin_name.to_string(),
                marketplace: marketplace_name.to_string(),
            })
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

        let version = self.read_plugin_version(&plugin_path);

        Ok(ResolvedPlugin {
            name: plugin_name.to_string(),
            marketplace: marketplace_name.to_string(),
            source_type: SourceType::Local,
            marketplace_commit: marketplace_commit.to_string(),
            plugin_commit: marketplace_commit.to_string(),
            resolved_version: version,
            source: path.to_string(),
        })
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

        // Read version from plugin.json
        let workdir = repo.workdir().unwrap_or(&plugin_cache_path);
        let version = self.read_plugin_version(workdir);

        Ok(ResolvedPlugin {
            name: plugin_name.to_string(),
            marketplace: marketplace_name.to_string(),
            source_type: SourceType::External,
            marketplace_commit: marketplace_commit.to_string(),
            plugin_commit,
            resolved_version: version,
            source: url.to_string(),
        })
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

    /// Read the version from plugin.json, returning "unknown" if not found.
    fn read_plugin_version(&self, plugin_path: &Path) -> String {
        let json_path = plugin_path.join("plugin.json");
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(plugin_json) = serde_json::from_str::<PluginJson>(&content) {
                if let Some(version) = plugin_json.version {
                    return version;
                }
            }
        }
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_marketplace_with_local_plugin(dir: &Path) -> Repository {
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

        // Create local plugin with plugin.json
        let plugin_dir = dir.join("plugins/local-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        let plugin_json = r#"{
            "name": "local-plugin",
            "version": "1.2.3",
            "description": "Test local plugin"
        }"#;
        fs::write(plugin_dir.join("plugin.json"), plugin_json).unwrap();

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

        let repo = setup_marketplace_with_local_plugin(&marketplace_dir);
        let commit = repo.head().unwrap().peel_to_commit().unwrap().id().to_string();

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let plugin_info = super::super::marketplace::MarketplacePlugin {
            path: Some("plugins/local-plugin".to_string()),
            url: None,
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
        assert_eq!(resolved.resolved_version, "1.2.3");
        assert_eq!(resolved.source, "plugins/local-plugin");
    }

    #[test]
    fn test_read_plugin_version_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());

        // No plugin.json exists
        let version = resolver.read_plugin_version(temp_dir.path());
        assert_eq!(version, "unknown");
    }

    #[test]
    fn test_read_plugin_version_no_version_field() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("plugin.json"),
            r#"{"name": "test"}"#,
        )
        .unwrap();

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let version = resolver.read_plugin_version(temp_dir.path());
        assert_eq!(version, "unknown");
    }
}
