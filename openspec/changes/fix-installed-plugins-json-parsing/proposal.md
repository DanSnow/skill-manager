## Why

skill-manager fails to parse Claude Code's `~/.claude/plugins/installed_plugins.json` because the code expects a flat `HashMap<String, InstalledPlugin>` but the actual file uses a v2 format with a wrapper object (`version` + `plugins`) and arrays per plugin key to support multiple installations (user-scope and project-scope).

## What Changes

- **BREAKING**: Change `read_installed_plugins()` return type from `HashMap<String, InstalledPlugin>` to `InstalledPluginsFile` struct
- Add `InstalledPluginsFile` wrapper struct with `version: u32` and `plugins: HashMap<String, Vec<InstalledPluginEntry>>`
- Add `project_path: Option<String>` field to `InstalledPluginEntry` for project-scoped plugins
- Add `PluginScope` enum (`User` | `Project(PathBuf)`) to represent installation scope
- Update `add_installed_plugin()` to accept scope parameter and preserve entries with different scopes
- Update install command to determine scope from manifest location and pass it to Claude integration

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `plugin-installation`: Add support for project-scoped plugin installations with proper scope detection from manifest location, and preserve existing entries with different scopes when updating

## Impact

- `src/installer/claude.rs`: Major changes to structs and methods
- `src/cli/install.rs`: Pass scope to `add_installed_plugin()`
- Tests in `claude.rs` need updating for new struct shapes
