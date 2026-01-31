use std::cell::OnceCell;
use std::path::{Path, PathBuf};

/// Encapsulates the Claude plugin directory structure conventions.
///
/// Provides lazy-cached path accessors for common plugin file locations:
/// - `.claude-plugin/` - config directory
/// - `.claude-plugin/plugin.json` - plugin metadata
/// - `.claude-plugin/marketplace.json` - marketplace listing entry
#[derive(Debug)]
pub struct PluginLayout {
    base_path: PathBuf,
    config_dir: OnceCell<PathBuf>,
    plugin_json: OnceCell<PathBuf>,
    marketplace_json: OnceCell<PathBuf>,
}

impl Clone for PluginLayout {
    fn clone(&self) -> Self {
        // Clone the base_path, create fresh cells (paths will be recomputed lazily)
        Self::new(self.base_path.clone())
    }
}

impl PluginLayout {
    /// Create a new PluginLayout for the given base path.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            config_dir: OnceCell::new(),
            plugin_json: OnceCell::new(),
            marketplace_json: OnceCell::new(),
        }
    }

    /// Returns reference to the base path.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Returns reference to the .claude-plugin directory path.
    pub fn config_dir(&self) -> &Path {
        self.config_dir
            .get_or_init(|| self.base_path.join(".claude-plugin"))
    }

    /// Returns reference to the .claude-plugin/plugin.json path.
    pub fn plugin_json(&self) -> &Path {
        self.plugin_json
            .get_or_init(|| self.config_dir().join("plugin.json"))
    }

    /// Returns reference to the .claude-plugin/marketplace.json path.
    pub fn marketplace_json(&self) -> &Path {
        self.marketplace_json
            .get_or_init(|| self.config_dir().join("marketplace.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_path() {
        let layout = PluginLayout::new("/path/to/plugin");
        assert_eq!(layout.base_path(), Path::new("/path/to/plugin"));
    }

    #[test]
    fn test_config_dir() {
        let layout = PluginLayout::new("/path/to/plugin");
        assert_eq!(
            layout.config_dir(),
            Path::new("/path/to/plugin/.claude-plugin")
        );
    }

    #[test]
    fn test_plugin_json() {
        let layout = PluginLayout::new("/path/to/plugin");
        assert_eq!(
            layout.plugin_json(),
            Path::new("/path/to/plugin/.claude-plugin/plugin.json")
        );
    }

    #[test]
    fn test_marketplace_json() {
        let layout = PluginLayout::new("/path/to/plugin");
        assert_eq!(
            layout.marketplace_json(),
            Path::new("/path/to/plugin/.claude-plugin/marketplace.json")
        );
    }

    #[test]
    fn test_paths_are_cached() {
        let layout = PluginLayout::new("/path/to/plugin");

        // Call twice to verify caching works (same reference returned)
        let config1 = layout.config_dir();
        let config2 = layout.config_dir();
        assert!(std::ptr::eq(config1, config2));

        let plugin1 = layout.plugin_json();
        let plugin2 = layout.plugin_json();
        assert!(std::ptr::eq(plugin1, plugin2));

        let marketplace1 = layout.marketplace_json();
        let marketplace2 = layout.marketplace_json();
        assert!(std::ptr::eq(marketplace1, marketplace2));
    }
}
