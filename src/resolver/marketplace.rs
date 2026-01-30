use git2::{FetchOptions, RemoteCallbacks, Repository};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument, trace};

use crate::{Error, Result};

/// Metadata for a plugin entry in marketplace.json.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketplacePlugin {
    /// Local path within marketplace (for local plugins).
    pub path: Option<String>,
    /// Git URL for external plugin repository.
    pub url: Option<String>,
    /// Optional description.
    pub description: Option<String>,
}

/// Parsed marketplace.json structure.
#[derive(Debug, Clone, Deserialize)]
pub struct MarketplaceJson {
    pub plugins: HashMap<String, MarketplacePlugin>,
}

/// Operations for working with marketplace git repositories.
pub struct MarketplaceResolver {
    pub(crate) cache_dir: PathBuf,
}

impl MarketplaceResolver {
    /// Create a new resolver with the given cache directory.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the local path for a marketplace.
    pub fn marketplace_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join("marketplaces").join(name)
    }

    /// Clone or fetch a marketplace repository.
    #[instrument(skip(self), fields(path))]
    pub fn ensure_marketplace(&self, name: &str, url: &str) -> Result<Repository> {
        let path = self.marketplace_path(name);
        tracing::Span::current().record("path", path.display().to_string());

        if path.exists() {
            debug!("marketplace exists locally, fetching updates");
            self.fetch_marketplace(name, &path)
        } else {
            debug!("marketplace not found locally, cloning");
            self.clone_marketplace(name, url, &path)
        }
    }

    /// Clone a marketplace to the cache.
    #[instrument(skip(self))]
    fn clone_marketplace(&self, name: &str, url: &str, path: &Path) -> Result<Repository> {
        debug!(path = %path.display(), "creating cache directory");
        std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
            .map_err(Error::CacheCreate)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            // Try SSH agent first for git@ URLs
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    return git2::Cred::ssh_key_from_agent(username);
                }
            }
            // Fall back to default credentials
            git2::Cred::default()
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        builder.clone(url, path).map_err(|e| Error::MarketplaceClone {
            name: name.to_string(),
            source: e,
        })
    }

    /// Fetch updates for an existing marketplace clone.
    #[instrument(skip(self))]
    fn fetch_marketplace(&self, name: &str, path: &Path) -> Result<Repository> {
        debug!(path = %path.display(), "opening existing repository");
        let repo = Repository::open(path).map_err(|e| Error::MarketplaceClone {
            name: name.to_string(),
            source: e,
        })?;

        {
            let mut remote = repo.find_remote("origin").map_err(|e| Error::MarketplaceFetch {
                name: name.to_string(),
                source: e,
            })?;

            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, allowed_types| {
                if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                    if let Some(username) = username_from_url {
                        return git2::Cred::ssh_key_from_agent(username);
                    }
                }
                git2::Cred::default()
            });

            let mut fo = FetchOptions::new();
            fo.remote_callbacks(callbacks);

            remote
                .fetch(&["refs/heads/*:refs/heads/*", "refs/tags/*:refs/tags/*"], Some(&mut fo), None)
                .map_err(|e| Error::MarketplaceFetch {
                    name: name.to_string(),
                    source: e,
                })?;
        }

        Ok(repo)
    }

    /// Resolve a tag to its commit hash.
    pub fn resolve_tag(&self, repo: &Repository, marketplace: &str, tag: &str) -> Result<String> {
        let refname = format!("refs/tags/{}", tag);
        let reference = repo.find_reference(&refname).map_err(|_| Error::TagNotFound {
            marketplace: marketplace.to_string(),
            tag: tag.to_string(),
        })?;

        // Handle both lightweight and annotated tags
        let commit = reference.peel_to_commit().map_err(|_| Error::TagNotFound {
            marketplace: marketplace.to_string(),
            tag: tag.to_string(),
        })?;

        Ok(commit.id().to_string())
    }

    /// Resolve HEAD to its commit hash.
    pub fn resolve_head(&self, repo: &Repository) -> Result<String> {
        let head = repo.head().map_err(Error::Git)?;
        let commit = head.peel_to_commit().map_err(Error::Git)?;
        Ok(commit.id().to_string())
    }

    /// Checkout a specific commit.
    #[instrument(skip(self, repo))]
    pub fn checkout_commit(&self, repo: &Repository, marketplace: &str, commit: &str) -> Result<()> {
        debug!("parsing commit oid");
        let oid = git2::Oid::from_str(commit).map_err(|_| Error::CommitNotFound {
            marketplace: marketplace.to_string(),
            commit: commit.to_string(),
        })?;

        debug!("finding commit object");
        let commit_obj = repo.find_commit(oid).map_err(|_| Error::CommitNotFound {
            marketplace: marketplace.to_string(),
            commit: commit.to_string(),
        })?;

        debug!("checking out tree");
        repo.checkout_tree(commit_obj.as_object(), Some(git2::build::CheckoutBuilder::new().force()))
            .map_err(Error::Git)?;

        debug!("setting HEAD to detached state");
        repo.set_head_detached(oid).map_err(Error::Git)?;

        debug!("checkout complete");
        Ok(())
    }

    /// Parse marketplace.json from a repository.
    #[instrument(skip(self, repo))]
    pub fn parse_marketplace_json(&self, repo: &Repository, marketplace: &str) -> Result<MarketplaceJson> {
        let workdir = repo.workdir().ok_or_else(|| {
            debug!("repository has no workdir (bare repository?)");
            Error::MarketplaceJsonNotFound(marketplace.to_string())
        })?;

        let json_path = workdir.join(".claude-plugin").join("marketplace.json");
        debug!(path = %json_path.display(), "looking for marketplace.json");

        if !json_path.exists() {
            debug!(path = %json_path.display(), "marketplace.json not found");
            return Err(Error::MarketplaceJsonNotFound(marketplace.to_string()));
        }

        debug!("reading marketplace.json");
        let content = std::fs::read_to_string(&json_path).map_err(|e| Error::FileRead {
            path: json_path.clone(),
            source: e,
        })?;
        trace!(content_len = content.len(), "marketplace.json content loaded");

        debug!("parsing marketplace.json");
        let parsed: MarketplaceJson = serde_json::from_str(&content).map_err(|e| {
            debug!(error = %e, "failed to parse marketplace.json");
            Error::MarketplaceJsonParse {
                name: marketplace.to_string(),
                reason: e.to_string(),
            }
        })?;

        debug!(plugin_count = parsed.plugins.len(), "marketplace.json parsed successfully");
        trace!(plugins = ?parsed.plugins.keys().collect::<Vec<_>>(), "available plugins");

        Ok(parsed)
    }

    /// Find a plugin in a marketplace.
    #[instrument(skip(self, marketplace_json))]
    pub fn find_plugin<'a>(
        &self,
        marketplace_json: &'a MarketplaceJson,
        marketplace: &str,
        plugin: &str,
    ) -> Result<&'a MarketplacePlugin> {
        debug!("searching for plugin in marketplace.json");
        marketplace_json.plugins.get(plugin).ok_or_else(|| {
            debug!(
                available = ?marketplace_json.plugins.keys().collect::<Vec<_>>(),
                "plugin not found in marketplace"
            );
            Error::PluginNotFound {
                plugin: plugin.to_string(),
                marketplace: marketplace.to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_repo(dir: &Path) -> Repository {
        let repo = Repository::init(dir).unwrap();

        // Create .claude-plugins directory and marketplace.json
        let plugins_dir = dir.join(".claude-plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        let json_content = r#"{
            "plugins": {
                "test-plugin": {
                    "path": "plugins/test-plugin",
                    "description": "A test plugin"
                },
                "external-plugin": {
                    "url": "https://github.com/example/external.git",
                    "description": "An external plugin"
                }
            }
        }"#;
        fs::write(plugins_dir.join("marketplace.json"), json_content).unwrap();

        // Commit the file
        {
            let mut index = repo.index().unwrap();
            index.add_path(Path::new(".claude-plugins/marketplace.json")).unwrap();
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
    fn test_parse_marketplace_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(temp_dir.path());

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let json = resolver.parse_marketplace_json(&repo, "test").unwrap();

        assert!(json.plugins.contains_key("test-plugin"));
        assert!(json.plugins.contains_key("external-plugin"));

        let local = &json.plugins["test-plugin"];
        assert_eq!(local.path, Some("plugins/test-plugin".to_string()));
        assert!(local.url.is_none());

        let external = &json.plugins["external-plugin"];
        assert!(external.path.is_none());
        assert_eq!(
            external.url,
            Some("https://github.com/example/external.git".to_string())
        );
    }

    #[test]
    fn test_resolve_head() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(temp_dir.path());

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let commit = resolver.resolve_head(&repo).unwrap();

        assert!(!commit.is_empty());
        assert_eq!(commit.len(), 40); // SHA-1 hex length
    }

    #[test]
    fn test_find_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(temp_dir.path());

        let resolver = MarketplaceResolver::new(temp_dir.path().to_path_buf());
        let json = resolver.parse_marketplace_json(&repo, "test").unwrap();

        let plugin = resolver.find_plugin(&json, "test", "test-plugin").unwrap();
        assert_eq!(plugin.path, Some("plugins/test-plugin".to_string()));

        let result = resolver.find_plugin(&json, "test", "nonexistent");
        assert!(matches!(result, Err(Error::PluginNotFound { .. })));
    }
}
