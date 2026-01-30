## Context

Claude Code's `installed_plugins.json` uses a v2 format that differs from what skill-manager expects:

**Actual format:**
```json
{
  "version": 2,
  "plugins": {
    "plugin@marketplace": [
      { "scope": "user", "installPath": "...", ... },
      { "scope": "project", "projectPath": "/path", ... }
    ]
  }
}
```

**Current code expects:**
```json
{
  "plugin@marketplace": { "scope": "user", ... }
}
```

The array-per-plugin design allows the same plugin to be installed at user scope (global) and multiple project scopes (per-project). skill-manager needs to support both scopes and preserve entries it doesn't own.

## Goals / Non-Goals

**Goals:**
- Parse the actual v2 format with wrapper object and arrays
- Support user-scope installations (from global manifest)
- Support project-scope installations (from project manifest)
- Preserve existing entries with different scopes when updating
- Use canonicalized absolute paths for projectPath

**Non-Goals:**
- Supporting v1 format migration (assume v2 everywhere)
- Managing entries not tracked in skill-manager's lockfile

## Decisions

### Decision: Add wrapper struct for file format

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledPluginsFile {
    pub version: u32,
    pub plugins: HashMap<String, Vec<InstalledPluginEntry>>,
}
```

**Rationale:** Matches the actual JSON structure. Using `u32` for version allows future format changes.

### Decision: Use enum for scope instead of string matching

```rust
pub enum PluginScope {
    User,
    Project(PathBuf),
}
```

**Rationale:** Type-safe scope handling prevents bugs from string typos. The `PathBuf` carries the project directory for project-scoped installs.

### Decision: Determine scope from manifest location

- Global manifest (`~/.config/skill-manager/plugins.toml`) → `PluginScope::User`
- Project manifest (`./.claude/plugins.toml`) → `PluginScope::Project(canonicalize(cwd))`

**Rationale:** Implicit from file location matches user mental model. No extra CLI flags needed.

### Decision: Filter-and-replace update strategy

When adding a plugin entry:
1. Get or create the array for the plugin key
2. Remove any existing entry that matches the scope being installed
3. Add the new entry
4. Preserve all entries with different scopes

**Rationale:** Safe update that won't clobber project-scoped entries when updating user-scoped (and vice versa).

## Risks / Trade-offs

**[Risk] Canonicalization may fail on non-existent paths** → Use `std::fs::canonicalize` which requires the path to exist. Since we're running from the project directory, this should always succeed. If it fails, propagate the error.

**[Trade-off] No v1 migration** → Simplifies implementation. v1 format is likely rare in the wild since Claude Code has been on v2 for a while.

**[Trade-off] Hard-coded version = 2** → When writing, always output version 2. If Claude Code changes format in the future, we'll need to update.
