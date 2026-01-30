use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Entry in installed_plugins.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPlugin {
    pub scope: String,
    pub install_path: String,
    pub version: String,
    pub installed_at: String,
    pub last_updated: String,
    pub git_commit_sha: String,
}

/// Manages Claude Code's configuration files.
pub struct ClaudeCodeIntegration {
    claude_dir: PathBuf,
}

impl ClaudeCodeIntegration {
    /// Create a new integration using the default ~/.claude directory.
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            claude_dir: PathBuf::from(home).join(".claude"),
        }
    }

    /// Create an integration with a custom Claude directory (for testing).
    pub fn with_claude_dir(claude_dir: PathBuf) -> Self {
        Self { claude_dir }
    }

    /// Get the path to installed_plugins.json.
    pub fn installed_plugins_path(&self) -> PathBuf {
        self.claude_dir.join("plugins").join("installed_plugins.json")
    }

    /// Get the path to settings.json.
    pub fn settings_path(&self) -> PathBuf {
        self.claude_dir.join("settings.json")
    }

    /// Read existing installed_plugins.json or return empty map.
    pub fn read_installed_plugins(&self) -> Result<HashMap<String, InstalledPlugin>> {
        let path = self.installed_plugins_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(&path).map_err(|e| Error::FileRead {
            path: path.clone(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|e| Error::JsonParse {
            path,
            source: e,
        })
    }

    /// Write installed_plugins.json.
    pub fn write_installed_plugins(&self, plugins: &HashMap<String, InstalledPlugin>) -> Result<()> {
        let path = self.installed_plugins_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::FileWrite {
                path: path.clone(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(plugins).map_err(|e| Error::JsonParse {
            path: path.clone(),
            source: e,
        })?;

        std::fs::write(&path, content).map_err(|e| Error::FileWrite {
            path,
            source: e,
        })
    }

    /// Add or update a plugin in installed_plugins.json.
    pub fn add_installed_plugin(
        &self,
        plugin_name: &str,
        marketplace: &str,
        install_path: &Path,
        version: &str,
        commit: &str,
    ) -> Result<()> {
        let mut plugins = self.read_installed_plugins()?;

        let key = format!("{}@{}", plugin_name, marketplace);
        let now = chrono_iso8601_now();

        plugins.insert(
            key,
            InstalledPlugin {
                scope: "user".to_string(),
                install_path: install_path.to_string_lossy().to_string(),
                version: version.to_string(),
                installed_at: now.clone(),
                last_updated: now,
                git_commit_sha: commit.to_string(),
            },
        );

        self.write_installed_plugins(&plugins)
    }

    /// Read existing settings.json or return empty object.
    pub fn read_settings(&self) -> Result<Map<String, Value>> {
        let path = self.settings_path();
        if !path.exists() {
            return Ok(Map::new());
        }

        let content = std::fs::read_to_string(&path).map_err(|e| Error::FileRead {
            path: path.clone(),
            source: e,
        })?;

        let value: Value = serde_json::from_str(&content).map_err(|e| Error::JsonParse {
            path,
            source: e,
        })?;

        match value {
            Value::Object(map) => Ok(map),
            _ => Ok(Map::new()),
        }
    }

    /// Write settings.json.
    pub fn write_settings(&self, settings: &Map<String, Value>) -> Result<()> {
        let path = self.settings_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::FileWrite {
                path: path.clone(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(&Value::Object(settings.clone()))
            .map_err(|e| Error::JsonParse {
                path: path.clone(),
                source: e,
            })?;

        std::fs::write(&path, content).map_err(|e| Error::FileWrite {
            path,
            source: e,
        })
    }

    /// Enable a plugin in settings.json.
    pub fn enable_plugin(&self, plugin_name: &str, marketplace: &str) -> Result<()> {
        let mut settings = self.read_settings()?;

        let key = format!("{}@{}", plugin_name, marketplace);

        // Get or create enabledPlugins
        let enabled_plugins = settings
            .entry("enabledPlugins")
            .or_insert_with(|| json!({}));

        if let Value::Object(map) = enabled_plugins {
            map.insert(key, json!(true));
        }

        self.write_settings(&settings)
    }
}

/// Get current time in ISO 8601 format.
fn chrono_iso8601_now() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();

    // Simple ISO 8601 format without external crate
    // This is approximate but sufficient for our needs
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    // Calculate year/month/day (simplified, not accounting for leap years perfectly)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch as i64;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months: [i64; 12] = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months {
        if remaining_days < days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_installed_plugins_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        let plugins = integration.read_installed_plugins().unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_read_installed_plugins_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        let content = r#"{
            "superpowers@official": {
                "scope": "user",
                "installPath": "/path/to/plugin",
                "version": "4.1.1",
                "installedAt": "2024-01-01T00:00:00Z",
                "lastUpdated": "2024-01-01T00:00:00Z",
                "gitCommitSha": "abc123"
            }
        }"#;
        fs::write(plugins_dir.join("installed_plugins.json"), content).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());
        let plugins = integration.read_installed_plugins().unwrap();

        assert_eq!(plugins.len(), 1);
        assert!(plugins.contains_key("superpowers@official"));
        assert_eq!(plugins["superpowers@official"].version, "4.1.1");
    }

    #[test]
    fn test_add_installed_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin"),
                "1.0.0",
                "abc123",
            )
            .unwrap();

        let plugins = integration.read_installed_plugins().unwrap();
        assert_eq!(plugins.len(), 1);
        assert!(plugins.contains_key("test-plugin@official"));
        assert_eq!(plugins["test-plugin@official"].version, "1.0.0");
        assert_eq!(plugins["test-plugin@official"].git_commit_sha, "abc123");
    }

    #[test]
    fn test_read_settings_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        let settings = integration.read_settings().unwrap();
        assert!(settings.is_empty());
    }

    #[test]
    fn test_enable_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        integration.enable_plugin("superpowers", "official").unwrap();

        let settings = integration.read_settings().unwrap();
        assert!(settings.contains_key("enabledPlugins"));

        let enabled = settings["enabledPlugins"].as_object().unwrap();
        assert_eq!(enabled["superpowers@official"], json!(true));
    }

    #[test]
    fn test_enable_plugin_preserves_existing() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create existing settings
        let existing = r#"{
            "someOtherSetting": "value",
            "enabledPlugins": {
                "existing@marketplace": true
            }
        }"#;
        fs::write(temp_dir.path().join("settings.json"), existing).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());
        integration.enable_plugin("new-plugin", "official").unwrap();

        let settings = integration.read_settings().unwrap();

        // Check existing setting preserved
        assert_eq!(settings["someOtherSetting"], "value");

        // Check both plugins enabled
        let enabled = settings["enabledPlugins"].as_object().unwrap();
        assert_eq!(enabled["existing@marketplace"], json!(true));
        assert_eq!(enabled["new-plugin@official"], json!(true));
    }

    #[test]
    fn test_chrono_iso8601_now() {
        let timestamp = chrono_iso8601_now();
        // Should be in format YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(timestamp.len(), 20);
        assert!(timestamp.ends_with('Z'));
        assert!(timestamp.contains('T'));
    }
}
