## Context

Currently, `MarketplaceResolver::read_plugin_version()` in `src/resolver/plugin.rs` looks for `plugin.json` at the plugin's root directory. However, Claude Code plugins store their metadata in `.claude-plugin/plugin.json`. This mismatch causes all version lookups to fail, resulting in "unknown" being written to `plugins.lock`.

The codebase has several places that need to understand the Claude plugin directory structure:
- Plugin version reading (currently broken)
- Marketplace JSON parsing (in `src/resolver/marketplace.rs`)
- Potentially future operations on plugin files

## Goals / Non-Goals

**Goals:**
- Fix version resolution to read from the correct path (`.claude-plugin/plugin.json`)
- Provide a reusable abstraction for Claude plugin directory structure
- Add meaningful fallback (git SHA) when version is truly unavailable
- Keep the abstraction simple and focused

**Non-Goals:**
- Full plugin validation or schema enforcement
- Caching or performance optimization for file reads
- Supporting non-standard plugin layouts

## Decisions

### 1. Introduce `PluginLayout` struct in new `src/layout.rs`

**Decision**: Create a dedicated module for the layout abstraction rather than embedding it in the resolver.

**Rationale**:
- Separation of concerns - path logic is distinct from resolution logic
- Reusable across multiple modules (resolver, future install/uninstall logic)
- Easy to test in isolation

**Alternatives considered**:
- Inline in `plugin.rs` - rejected because it limits reuse and mixes concerns
- Constants only - rejected because methods provide better ergonomics

### 2. `PluginLayout` API design

```rust
use std::cell::OnceCell;

pub struct PluginLayout {
    base_path: PathBuf,
    config_dir: OnceCell<PathBuf>,
    plugin_json: OnceCell<PathBuf>,
    marketplace_json: OnceCell<PathBuf>,
}

impl PluginLayout {
    pub fn new(base_path: impl Into<PathBuf>) -> Self;

    /// Returns reference to base path
    pub fn base_path(&self) -> &Path;

    /// Returns reference to .claude-plugin directory path
    pub fn config_dir(&self) -> &Path;

    /// Returns reference to .claude-plugin/plugin.json path
    pub fn plugin_json(&self) -> &Path;

    /// Returns reference to .claude-plugin/marketplace.json path
    pub fn marketplace_json(&self) -> &Path;
}
```

**Rationale**:
- All accessors return `&Path` - zero allocation after first access
- `OnceCell` for lazy initialization - paths computed only when needed
- `base_path` stored directly (always needed), others lazily cached
- Simple path accessors - no I/O in the struct itself
- Reading/parsing remains in caller code to maintain single responsibility

**Alternatives considered**:
- Eager computation of all paths - rejected because some paths may never be accessed
- Return `PathBuf` - rejected because it allocates on every call
- Include `read_plugin_json()` method - rejected to keep struct pure (no I/O)
- Use associated constants for directory name - could add later if needed

### 3. Version fallback to short git SHA

**Decision**: When `plugin.json` doesn't exist or has no version field, use the first 7 characters of `plugin_commit` as the resolved version.

**Rationale**:
- 7 chars is the standard git short SHA (enough to be unique in most repos)
- The commit SHA is always available during resolution
- Provides meaningful version info for debugging/inspection

**Format**: `27d2b86` (7 char SHA, no prefix)

**Alternatives considered**:
- Keep "unknown" - rejected because it provides no useful information
- Full SHA - rejected because it's too long for display
- Prefixed SHA like `sha:27d2b86` - rejected as unnecessary complexity

### 4. `ResolvedPlugin` owns its construction logic

**Decision**: Move version resolution into `ResolvedPlugin` with constructor methods:

```rust
impl ResolvedPlugin {
    /// Construct from a local plugin (within marketplace repo)
    pub fn from_local(
        name: String,
        marketplace: String,
        commit: String,
        source: String,
        layout: &PluginLayout,
    ) -> Self {
        let resolved_version = Self::read_version(layout)
            .unwrap_or_else(|| commit[..7].to_string());

        Self {
            name,
            marketplace,
            source_type: SourceType::Local,
            marketplace_commit: commit.clone(),
            plugin_commit: commit,
            resolved_version,
            source,
        }
    }

    /// Construct from an external plugin (separate git repo)
    pub fn from_external(
        name: String,
        marketplace: String,
        marketplace_commit: String,
        plugin_commit: String,
        source: String,
        layout: &PluginLayout,
    ) -> Self {
        let resolved_version = Self::read_version(layout)
            .unwrap_or_else(|| plugin_commit[..7].to_string());

        Self {
            name,
            marketplace,
            source_type: SourceType::External,
            marketplace_commit,
            plugin_commit,
            resolved_version,
            source,
        }
    }

    /// Read version from plugin.json, returns None if unavailable
    fn read_version(layout: &PluginLayout) -> Option<String> {
        let content = std::fs::read_to_string(layout.plugin_json()).ok()?;
        let json: PluginJson = serde_json::from_str(&content).ok()?;
        json.version
    }
}
```

**Rationale**:
- Idiomatic Rust: types own their construction and invariants
- `ResolvedPlugin` guarantees `resolved_version` is always meaningful by construction
- `read_version` is a private implementation detail
- Resolver becomes simpler - just calls `ResolvedPlugin::from_local(...)`
- Can't accidentally create an incomplete `ResolvedPlugin`

**Alternatives considered**:
- Keep `read_plugin_version` in resolver - rejected because it splits responsibility
- Return `Option` and let caller handle - rejected because `ResolvedPlugin` should enforce its own completeness

## Risks / Trade-offs

**Risk**: Other code may depend on "unknown" as a sentinel value.
→ **Mitigation**: Search codebase for "unknown" usage; none found that depends on this behavior.

**Risk**: Short SHA collision in very large repos.
→ **Mitigation**: 7 chars provides ~268 million combinations; acceptable for plugin versioning context.

**Trade-off**: Adding a new module increases code surface area.
→ **Accepted**: The abstraction provides clear value and is small (~30 lines).
