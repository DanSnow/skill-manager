## Why

Claude Code requires marketplace information in `~/.claude/plugins/known_marketplaces.json` to recognize and load plugins from custom marketplaces. Currently, skill-manager installs plugins to `installed_plugins.json` and enables them in `settings.json`, but does not register the marketplace itself. This causes Claude Code to ignore plugins from unknown marketplaces.

## What Changes

- Add a new method to `ClaudeCodeIntegration` that writes marketplace entries to `~/.claude/plugins/known_marketplaces.json`
- Call this method during plugin installation, after resolving each marketplace but before installing plugins
- The entry format matches Claude Code's expected structure with `source`, `installLocation`, and `lastUpdated` fields

## Capabilities

### New Capabilities
- `marketplace-registration`: Register marketplaces with Claude Code by writing to `known_marketplaces.json`, enabling Claude Code to discover and load plugins from custom marketplaces

### Modified Capabilities
- None

## Impact

- **Code**: `src/installer/claude.rs` (new method), `src/cli/install.rs` (call site)
- **Files created**: `~/.claude/plugins/known_marketplaces.json` (if not exists)
- **Dependencies**: None - uses existing serde_json and std::fs
- **Breaking changes**: None - additive change that improves compatibility
