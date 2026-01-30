use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// CACHEDIR.TAG content per https://bford.info/cachedir/
const CACHEDIR_TAG_CONTENT: &str = "Signature: 8a477f597d28d172789f06886806bc55\n\
# This file is a cache directory tag created by skill-manager.\n\
# For information about cache directory tags, see:\n\
#   https://bford.info/cachedir/\n";

/// Cache manager for skill-manager.
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager using XDG cache directory.
    pub fn new() -> Result<Self> {
        let dirs = xdg::BaseDirectories::with_prefix("skill-manager");
        let cache_dir = dirs
            .get_cache_home()
            .ok_or_else(|| Error::CacheCreate(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine cache directory",
            )))?;

        Ok(Self { cache_dir })
    }

    /// Create a cache manager with a custom cache directory (for testing).
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Ensure the cache directory exists and has a CACHEDIR.TAG.
    pub fn ensure_cache_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir).map_err(Error::CacheCreate)?;

        let tag_path = self.cache_dir.join("CACHEDIR.TAG");
        if !tag_path.exists() {
            std::fs::write(&tag_path, CACHEDIR_TAG_CONTENT).map_err(Error::CacheCreate)?;
        }

        // Create subdirectories
        std::fs::create_dir_all(self.cache_dir.join("marketplaces")).map_err(Error::CacheCreate)?;
        std::fs::create_dir_all(self.cache_dir.join("plugins")).map_err(Error::CacheCreate)?;

        Ok(())
    }

    /// Get the path for an extracted plugin.
    /// Format: ~/.cache/skill-manager/plugins/<marketplace>/<plugin>/<commit>/
    pub fn plugin_path(&self, marketplace: &str, plugin: &str, commit: &str) -> PathBuf {
        self.cache_dir
            .join("plugins")
            .join(marketplace)
            .join(plugin)
            .join(commit)
    }

    /// Check if a plugin is already extracted at the given commit.
    pub fn is_plugin_extracted(&self, marketplace: &str, plugin: &str, commit: &str) -> bool {
        let path = self.plugin_path(marketplace, plugin, commit);
        path.exists()
    }

    /// Extract a local plugin from a marketplace to the cache.
    pub fn extract_local_plugin(
        &self,
        marketplace_path: &Path,
        plugin_source_path: &str,
        marketplace: &str,
        plugin: &str,
        commit: &str,
    ) -> Result<PathBuf> {
        let target_path = self.plugin_path(marketplace, plugin, commit);

        if target_path.exists() {
            return Ok(target_path);
        }

        let source_path = marketplace_path.join(plugin_source_path);
        if !source_path.exists() {
            return Err(Error::PluginExtract(
                plugin.to_string(),
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Plugin source not found: {}", source_path.display()),
                ),
            ));
        }

        // Create target directory and copy contents
        std::fs::create_dir_all(&target_path).map_err(|e| Error::PluginExtract(plugin.to_string(), e))?;
        copy_dir_recursive(&source_path, &target_path)
            .map_err(|e| Error::PluginExtract(plugin.to_string(), e))?;

        Ok(target_path)
    }

    /// Copy an external plugin repository to the cache.
    pub fn extract_external_plugin(
        &self,
        repo_path: &Path,
        marketplace: &str,
        plugin: &str,
        commit: &str,
    ) -> Result<PathBuf> {
        let target_path = self.plugin_path(marketplace, plugin, commit);

        if target_path.exists() {
            return Ok(target_path);
        }

        // Create target directory and copy contents (excluding .git)
        std::fs::create_dir_all(&target_path).map_err(|e| Error::PluginExtract(plugin.to_string(), e))?;
        copy_dir_recursive_exclude_git(repo_path, &target_path)
            .map_err(|e| Error::PluginExtract(plugin.to_string(), e))?;

        Ok(target_path)
    }
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Recursively copy a directory, excluding .git.
fn copy_dir_recursive_exclude_git(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&name);

        // Skip .git directory
        if name == ".git" {
            continue;
        }

        if ty.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursive_exclude_git(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ensure_cache_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::with_cache_dir(temp_dir.path().to_path_buf());

        cache.ensure_cache_dir().unwrap();

        assert!(temp_dir.path().join("CACHEDIR.TAG").exists());
        assert!(temp_dir.path().join("marketplaces").exists());
        assert!(temp_dir.path().join("plugins").exists());

        let tag_content = fs::read_to_string(temp_dir.path().join("CACHEDIR.TAG")).unwrap();
        assert!(tag_content.starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
    }

    #[test]
    fn test_plugin_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::with_cache_dir(temp_dir.path().to_path_buf());

        let path = cache.plugin_path("official", "superpowers", "abc123");
        assert_eq!(
            path,
            temp_dir.path().join("plugins/official/superpowers/abc123")
        );
    }

    #[test]
    fn test_is_plugin_extracted() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::with_cache_dir(temp_dir.path().to_path_buf());

        assert!(!cache.is_plugin_extracted("test", "plugin", "abc123"));

        let path = cache.plugin_path("test", "plugin", "abc123");
        fs::create_dir_all(&path).unwrap();

        assert!(cache.is_plugin_extracted("test", "plugin", "abc123"));
    }

    #[test]
    fn test_extract_local_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::with_cache_dir(temp_dir.path().join("cache"));

        // Create a mock marketplace with a plugin
        let marketplace_path = temp_dir.path().join("marketplace");
        let plugin_src = marketplace_path.join("plugins/test-plugin");
        fs::create_dir_all(&plugin_src).unwrap();
        fs::write(plugin_src.join("plugin.json"), r#"{"version": "1.0"}"#).unwrap();
        fs::write(plugin_src.join("init.lua"), "-- test").unwrap();

        // Extract the plugin
        let result = cache
            .extract_local_plugin(
                &marketplace_path,
                "plugins/test-plugin",
                "official",
                "test-plugin",
                "abc123",
            )
            .unwrap();

        assert!(result.exists());
        assert!(result.join("plugin.json").exists());
        assert!(result.join("init.lua").exists());

        // Second extraction should return the same path (skip logic)
        let result2 = cache
            .extract_local_plugin(
                &marketplace_path,
                "plugins/test-plugin",
                "official",
                "test-plugin",
                "abc123",
            )
            .unwrap();
        assert_eq!(result, result2);
    }

    #[test]
    fn test_extract_external_plugin_excludes_git() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = CacheManager::with_cache_dir(temp_dir.path().join("cache"));

        // Create a mock repo with .git directory
        let repo_path = temp_dir.path().join("repo");
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        fs::write(repo_path.join(".git/config"), "git config").unwrap();
        fs::write(repo_path.join("plugin.json"), r#"{"version": "1.0"}"#).unwrap();

        // Extract the plugin
        let result = cache
            .extract_external_plugin(&repo_path, "test", "plugin", "def456")
            .unwrap();

        assert!(result.exists());
        assert!(result.join("plugin.json").exists());
        assert!(!result.join(".git").exists()); // .git should be excluded
    }
}
