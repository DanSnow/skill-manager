## 1. Update Data Structures

- [x] 1.1 Add `InstalledPluginsFile` wrapper struct with `version: u32` and `plugins: HashMap<String, Vec<InstalledPluginEntry>>`
- [x] 1.2 Rename `InstalledPlugin` to `InstalledPluginEntry` and add `project_path: Option<String>` field with serde skip_serializing_if
- [x] 1.3 Add `PluginScope` enum with `User` and `Project(PathBuf)` variants

## 2. Update Claude Integration Methods

- [x] 2.1 Update `read_installed_plugins()` to parse v2 format and return `InstalledPluginsFile`
- [x] 2.2 Update `write_installed_plugins()` to write v2 format wrapper
- [x] 2.3 Update `add_installed_plugin()` signature to accept `PluginScope` parameter
- [x] 2.4 Implement scope-aware filter-and-replace logic in `add_installed_plugin()`

## 3. Update Install Command

- [x] 3.1 Add helper function to determine `PluginScope` from manifest path
- [x] 3.2 Pass scope to `add_installed_plugin()` in install.rs

## 4. Update Tests

- [x] 4.1 Update `test_read_installed_plugins_existing` for v2 format with arrays
- [x] 4.2 Update `test_add_installed_plugin` for new signature and scope
- [x] 4.3 Add test for preserving entries with different scopes
- [x] 4.4 Add test for project-scope installation with projectPath
