## Why

Claude Code's plugin system lacks reproducibility. The internal files (`installed_plugins.json`, `known_marketplaces.json`, `settings.json`) are bookkeeping, not shareable manifests. You can't give a teammate a file and say "run this to get my exact setup."

## What Changes

- Add `plugins.toml` manifest file format for declaring desired plugins
- Add `plugins.lock` lock file for pinning exact git commits
- Add `skill-manager` CLI with commands: `init`, `add`, `install`, `remove`, `list`
- Write to Claude Code's `installed_plugins.json` and `settings.json` to integrate plugins
- Support both global (`~/.config/skill-manager/`) and project-local (`.claude/`) manifests
- Support GitHub shorthand, SSH URLs, and HTTPS URLs for marketplaces
- Handle conflicts between global and project plugins with user prompts

## Capabilities

### New Capabilities

- `manifest-format`: TOML-based manifest (`plugins.toml`) and lock file (`plugins.lock`) formats for declaring and pinning plugin versions
- `cli-commands`: Command-line interface with `init`, `add`, `install`, `remove`, and `list` commands
- `marketplace-resolution`: Clone and manage marketplace repositories, resolve plugins from marketplaces
- `plugin-installation`: Extract plugins to cache, write to Claude Code's JSON files for integration
- `conflict-handling`: Detect and resolve conflicts between global and project-level plugins

### Modified Capabilities

(none - this is a new project)

## Impact

- **New binary**: `skill-manager` Rust CLI
- **File system**: Creates `~/.config/skill-manager/` and `~/.cache/skill-manager/` directories
- **Claude Code integration**: Writes to `~/.claude/plugins/installed_plugins.json` and `~/.claude/settings.json`
- **Dependencies**: Rust crates: clap, toml, toml_edit, serde, serde_json, git2, xdg, thiserror, rootcause
