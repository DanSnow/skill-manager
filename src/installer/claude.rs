use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Represents the scope of a plugin installation.
#[derive(Debug, Clone)]
pub enum PluginScope {
    User,
    Project(PathBuf),
}

/// Wrapper for installed_plugins.json v2 format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPluginsFile {
    pub version: u32,
    pub plugins: HashMap<String, Vec<InstalledPluginEntry>>,
}

/// Entry in installed_plugins.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPluginEntry {
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
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

    /// Read existing installed_plugins.json or return empty v2 structure.
    pub fn read_installed_plugins(&self) -> Result<InstalledPluginsFile> {
        let path = self.installed_plugins_path();
        if !path.exists() {
            return Ok(InstalledPluginsFile {
                version: 2,
                plugins: HashMap::new(),
            });
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

    /// Write installed_plugins.json in v2 format.
    pub fn write_installed_plugins(&self, file: &InstalledPluginsFile) -> Result<()> {
        let path = self.installed_plugins_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::FileWrite {
                path: path.clone(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(file).map_err(|e| Error::JsonParse {
            path: path.clone(),
            source: e,
        })?;

        std::fs::write(&path, content).map_err(|e| Error::FileWrite {
            path,
            source: e,
        })
    }

    /// Add or update a plugin in installed_plugins.json.
    /// Uses scope-aware filter-and-replace: removes existing entries with the same scope,
    /// preserves entries with different scopes.
    pub fn add_installed_plugin(
        &self,
        plugin_name: &str,
        marketplace: &str,
        install_path: &Path,
        version: &str,
        commit: &str,
        scope: &PluginScope,
    ) -> Result<()> {
        let mut file = self.read_installed_plugins()?;

        let key = format!("{}@{}", plugin_name, marketplace);
        let now = chrono_iso8601_now();

        // Determine scope string and project_path based on PluginScope
        let (scope_str, project_path) = match scope {
            PluginScope::User => ("user".to_string(), None),
            PluginScope::Project(path) => {
                let canonical = std::fs::canonicalize(path).map_err(|e| Error::FileRead {
                    path: path.clone(),
                    source: e,
                })?;
                ("project".to_string(), Some(canonical.to_string_lossy().to_string()))
            }
        };

        let new_entry = InstalledPluginEntry {
            scope: scope_str.clone(),
            project_path: project_path.clone(),
            install_path: install_path.to_string_lossy().to_string(),
            version: version.to_string(),
            installed_at: now.clone(),
            last_updated: now,
            git_commit_sha: commit.to_string(),
        };

        // Get or create the array for this plugin key
        let entries = file.plugins.entry(key).or_insert_with(Vec::new);

        // Filter out existing entries with the same scope
        entries.retain(|entry| {
            if entry.scope != scope_str {
                return true; // Keep entries with different scope types
            }
            // For project scope, only remove if same project path
            if scope_str == "project" {
                return entry.project_path != project_path;
            }
            // For user scope, remove the existing entry
            false
        });

        // Add the new entry
        entries.push(new_entry);

        self.write_installed_plugins(&file)
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

/// Get current time in ISO 8601 format (UTC).
fn chrono_iso8601_now() -> String {
    use std::time::SystemTime;

    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_MINUTE: u64 = 60;

    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let days_since_epoch = secs / SECONDS_PER_DAY;
    let time_of_day = secs % SECONDS_PER_DAY;

    // Calculate year from days since epoch
    let (year, remaining_days) = year_from_days(days_since_epoch);

    // Calculate month and day
    let (month, day) = month_day_from_days(year, remaining_days);

    // Calculate time components
    let hours = time_of_day / SECONDS_PER_HOUR;
    let minutes = (time_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = time_of_day % SECONDS_PER_MINUTE;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Calculate year and remaining days from days since Unix epoch.
fn year_from_days(days_since_epoch: u64) -> (i32, u64) {
    let mut year = 1970;
    let mut remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            return (year, remaining);
        }
        remaining -= days_in_year;
        year += 1;
    }
}

/// Calculate month (1-12) and day (1-31) from year and day-of-year.
fn month_day_from_days(year: i32, day_of_year: u64) -> (u32, u32) {
    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining = day_of_year;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining < days {
            return ((i + 1) as u32, (remaining + 1) as u32);
        }
        remaining -= days;
    }

    // Fallback (should not reach)
    (12, 31)
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_installed_plugins_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        let file = integration.read_installed_plugins().unwrap();
        assert!(file.plugins.is_empty());
        assert_eq!(file.version, 2);
    }

    #[test]
    fn test_read_installed_plugins_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        // v2 format with wrapper and arrays
        let content = r#"{
            "version": 2,
            "plugins": {
                "superpowers@official": [
                    {
                        "scope": "user",
                        "installPath": "/path/to/plugin",
                        "version": "4.1.1",
                        "installedAt": "2024-01-01T00:00:00Z",
                        "lastUpdated": "2024-01-01T00:00:00Z",
                        "gitCommitSha": "abc123"
                    }
                ]
            }
        }"#;
        fs::write(plugins_dir.join("installed_plugins.json"), content).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());
        let file = integration.read_installed_plugins().unwrap();

        assert_eq!(file.version, 2);
        assert_eq!(file.plugins.len(), 1);
        assert!(file.plugins.contains_key("superpowers@official"));
        let entries = &file.plugins["superpowers@official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "4.1.1");
        assert_eq!(entries[0].scope, "user");
    }

    #[test]
    fn test_add_installed_plugin_user_scope() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin"),
                "1.0.0",
                "abc123",
                &PluginScope::User,
            )
            .unwrap();

        let file = integration.read_installed_plugins().unwrap();
        assert_eq!(file.version, 2);
        assert_eq!(file.plugins.len(), 1);
        assert!(file.plugins.contains_key("test-plugin@official"));

        let entries = &file.plugins["test-plugin@official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "1.0.0");
        assert_eq!(entries[0].git_commit_sha, "abc123");
        assert_eq!(entries[0].scope, "user");
        assert!(entries[0].project_path.is_none());
    }

    #[test]
    fn test_add_installed_plugin_project_scope() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("my-project");
        fs::create_dir_all(&project_dir).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin"),
                "1.0.0",
                "abc123",
                &PluginScope::Project(project_dir.clone()),
            )
            .unwrap();

        let file = integration.read_installed_plugins().unwrap();
        let entries = &file.plugins["test-plugin@official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].scope, "project");
        assert!(entries[0].project_path.is_some());
        // The path should be canonicalized
        let canonical = fs::canonicalize(&project_dir).unwrap();
        assert_eq!(entries[0].project_path.as_ref().unwrap(), &canonical.to_string_lossy().to_string());
    }

    #[test]
    fn test_add_installed_plugin_preserves_different_scopes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("my-project");
        fs::create_dir_all(&project_dir).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        // First, add a user-scope entry
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/user-plugin"),
                "1.0.0",
                "user123",
                &PluginScope::User,
            )
            .unwrap();

        // Then add a project-scope entry for the same plugin
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/project-plugin"),
                "2.0.0",
                "project456",
                &PluginScope::Project(project_dir.clone()),
            )
            .unwrap();

        // Both entries should be preserved
        let file = integration.read_installed_plugins().unwrap();
        let entries = &file.plugins["test-plugin@official"];
        assert_eq!(entries.len(), 2);

        // Find user and project entries
        let user_entry = entries.iter().find(|e| e.scope == "user").unwrap();
        let project_entry = entries.iter().find(|e| e.scope == "project").unwrap();

        assert_eq!(user_entry.version, "1.0.0");
        assert_eq!(user_entry.git_commit_sha, "user123");
        assert!(user_entry.project_path.is_none());

        assert_eq!(project_entry.version, "2.0.0");
        assert_eq!(project_entry.git_commit_sha, "project456");
        assert!(project_entry.project_path.is_some());
    }

    #[test]
    fn test_add_installed_plugin_replaces_same_scope() {
        let temp_dir = tempfile::tempdir().unwrap();
        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        // Add user-scope entry
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin-v1"),
                "1.0.0",
                "commit1",
                &PluginScope::User,
            )
            .unwrap();

        // Update user-scope entry (should replace, not add)
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin-v2"),
                "2.0.0",
                "commit2",
                &PluginScope::User,
            )
            .unwrap();

        let file = integration.read_installed_plugins().unwrap();
        let entries = &file.plugins["test-plugin@official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "2.0.0");
        assert_eq!(entries[0].git_commit_sha, "commit2");
    }

    #[test]
    fn test_add_installed_plugin_replaces_same_project_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("my-project");
        fs::create_dir_all(&project_dir).unwrap();

        let integration = ClaudeCodeIntegration::with_claude_dir(temp_dir.path().to_path_buf());

        // Add project-scope entry
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin-v1"),
                "1.0.0",
                "commit1",
                &PluginScope::Project(project_dir.clone()),
            )
            .unwrap();

        // Update same project-scope entry (should replace, not add)
        integration
            .add_installed_plugin(
                "test-plugin",
                "official",
                Path::new("/path/to/plugin-v2"),
                "2.0.0",
                "commit2",
                &PluginScope::Project(project_dir.clone()),
            )
            .unwrap();

        let file = integration.read_installed_plugins().unwrap();
        let entries = &file.plugins["test-plugin@official"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "2.0.0");
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
