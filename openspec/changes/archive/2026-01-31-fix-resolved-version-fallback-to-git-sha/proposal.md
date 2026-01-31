## Why

The `resolved_version` field in `plugins.lock` always shows "unknown" even when plugins have valid version information. This is caused by two issues:
1. The code looks for `plugin.json` at the wrong path (root instead of `.claude-plugin/plugin.json`)
2. When version truly isn't available, there's no fallback to use the git SHA as a meaningful identifier

## What Changes

- Introduce a `PluginLayout` struct that encapsulates Claude plugin/marketplace file structure conventions
  - Holds a base path
  - Provides methods: `marketplace_json_path()`, `plugin_json_path()`, etc.
  - Centralizes the `.claude-plugin/` directory convention
- Use `PluginLayout` in version resolution to correctly locate `plugin.json`
- Add fallback: when no version is found in plugin.json, use the git commit SHA (short form) as the resolved version
- Update tests to cover both the correct path lookup and SHA fallback behavior

## Capabilities

### New Capabilities
- `plugin-layout`: A struct abstracting Claude plugin directory structure, providing path accessors and potentially helper methods for common operations.

### Modified Capabilities
None - no spec-level behavior changes to existing capabilities.

## Impact

- **New code**: `src/layout.rs` (or similar) - `PluginLayout` struct
- **Modified code**: `src/resolver/plugin.rs` - use `PluginLayout` for path resolution, add SHA fallback
- **Lockfile**: `plugins.lock` will now show actual versions (e.g., "1.0.0") or short SHAs (e.g., "27d2b86") instead of "unknown"
- **User experience**: Users will see meaningful version information when inspecting their lockfile
- **Future**: Other code dealing with plugin paths can use `PluginLayout` for consistency
