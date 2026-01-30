## Context

Claude Code stores plugin state in three JSON files (`installed_plugins.json`, `known_marketplaces.json`, `settings.json`) that are bookkeeping rather than declarative manifests. Teams cannot share reproducible plugin setups because there's no way to express "install these exact plugins at these exact versions."

This project creates `skill-manager`, a standalone Rust CLI that introduces a Cargo-like workflow: a human-editable `plugins.toml` manifest and an auto-generated `plugins.lock` that pins exact git commits.

## Goals / Non-Goals

**Goals:**
- Reproducible plugin installations via lock file
- Human-readable manifest format (TOML)
- Support global and project-local configurations
- Integrate with Claude Code by writing to its JSON files
- Support multiple marketplace sources (GitHub shorthand, SSH, HTTPS URLs)

**Non-Goals:**
- Semantic version resolution (uses git tags/commits only)
- Modifying Claude Code's marketplace/cache folders
- Building a plugin browser or search UI
- Plugin development tooling

## Decisions

### 1. TOML for manifest, TOML for lock file

**Decision**: Use TOML for both `plugins.toml` and `plugins.lock`.

**Rationale**: TOML is human-readable, widely used in Rust ecosystem (Cargo.toml), and preserves comments. Using TOML for the lock file (vs JSON) maintains consistency and allows header comments explaining the file's purpose.

**Alternatives considered**:
- JSON: Less readable, no comments
- YAML: More ambiguous parsing rules

### 2. Git-based version pinning only

**Decision**: Pin versions by git tag or commit hash, not semver resolution.

**Rationale**: Marketplaces are git repositories. Tags map directly to commits. This avoids complex version constraint solving while still providing reproducibility. The `resolved_version` field is display-only, read from plugin.json.

**Alternatives considered**:
- Semver resolution: Adds complexity, marketplaces don't follow semver consistently

### 3. Separate cache from Claude Code's storage

**Decision**: Store cloned marketplaces and extracted plugins in `~/.cache/skill-manager/`, not in Claude Code's `~/.claude/plugins/` directories.

**Rationale**: Non-destructive design. skill-manager can coexist with Claude Code's built-in `/plugin install` command. Each tool manages its own cache. We only write to Claude Code's JSON files to register plugins.

**Alternatives considered**:
- Share Claude Code's cache: Risk of conflicts, harder to reason about state

### 4. XDG Base Directory Specification

**Decision**: Use XDG paths: `~/.config/skill-manager/` for config, `~/.cache/skill-manager/` for cache.

**Rationale**: Standard on Linux, works on macOS. The `xdg` crate handles platform differences. Clear separation between user configuration (tracked) and regenerable cache (not tracked).

### 5. Crate structure with clear boundaries

**Decision**: Organize into modules: `config/` (manifest/lock parsing), `resolver/` (git operations, marketplace handling), `installer/` (cache management, Claude Code integration), `cli/` (commands).

**Rationale**: Each module has a single responsibility. The resolver doesn't know about Claude Code. The installer doesn't know about TOML parsing. This enables testing modules in isolation.

## Risks / Trade-offs

**[Risk] Claude Code changes its JSON file formats** → We document which fields we write and monitor Claude Code releases. The integration layer (`installer/claude.rs`) is isolated for easy updates.

**[Risk] Git clone failures (network, auth)** → Surface clear error messages with the marketplace URL. Support SSH URLs for private repos.

**[Risk] Conflict between global and project plugins** → Prompt user with clear options: update global, skip, or abort. Don't silently override.

**[Trade-off] No semver resolution** → Simpler implementation but users must manually track compatible versions. This matches the MVP scope.

**[Trade-off] Separate cache from Claude Code** → Uses more disk space if same plugin installed via both tools. Acceptable for reliability.
