## Context

Claude Code discovers plugins through `~/.claude/plugins/known_marketplaces.json`. Each entry maps a marketplace name to its source and install location. Currently, skill-manager:

1. Clones marketplaces to `~/.cache/skill-manager/marketplaces/<name>/`
2. Extracts plugins to `~/.cache/skill-manager/plugins/<marketplace>/<plugin>/<commit>/`
3. Registers plugins in `~/.claude/plugins/installed_plugins.json`
4. Enables plugins in `~/.claude/settings.json`

Missing step: registering the marketplace itself in `known_marketplaces.json`.

**Constraints:**
- Must use `directory` source type so Claude Code doesn't try to manage/update the marketplace
- Must point to skill-manager's cache where the marketplace is actually cloned
- Must preserve existing entries in `known_marketplaces.json`
- Must match Claude Code's expected JSON structure exactly

## Goals / Non-Goals

**Goals:**
- Register marketplaces in `~/.claude/plugins/known_marketplaces.json` during install
- Use `directory` source type pointing to skill-manager's marketplace cache
- Prevent Claude Code from attempting to update skill-manager-managed marketplaces
- Preserve existing marketplace entries from other sources

**Non-Goals:**
- Parse or store GitHub repo information (not needed with directory source)
- Manage marketplace directory at `~/.claude/plugins/marketplaces/` - skill-manager uses its own cache
- Unregister marketplaces on plugin removal

## Decisions

### 1. Source type: directory for new entries

**Decision:** Use `"source": { "source": "directory", "path": "<cache-path>" }` when registering marketplaces via skill-manager.

**Rationale:**
- Points Claude Code to skill-manager's actual marketplace cache location
- Prevents Claude Code from trying to fetch/update the marketplace (skill-manager handles this)
- Simpler implementation - no URL parsing needed

**Alternatives considered:**
- Use `github` source type with repo info → Claude Code would try to manage updates, conflicting with skill-manager
- Use `url` source type → Same problem, Claude Code would fetch independently

### 2. Preserve unknown source types when reading

**Decision:** Use `serde_json::Value` for the `source` field to preserve any source type when reading/writing.

**Rationale:** The `known_marketplaces.json` file may contain entries created by Claude Code itself using `github`, `url`, or other source types. We must:
- Read these entries without failing on unknown variants
- Preserve them exactly when writing back
- Only create `directory` source for our own entries

**Alternatives considered:**
- Define all known source types as enum variants → Fragile, breaks if Claude Code adds new types
- Ignore/overwrite non-directory entries → Would corrupt user's Claude Code configuration

### 2. Install location path

**Decision:** Use skill-manager's marketplace cache path: `~/.cache/skill-manager/marketplaces/<marketplace-name>`

**Rationale:** This is where skill-manager actually clones the marketplace. Using the real path ensures Claude Code can find and read the marketplace contents.

**Alternatives considered:**
- Use `~/.claude/plugins/marketplaces/` → Would require copying/symlinking, adds complexity
- Different paths for source vs installLocation → Confusing, likely to cause issues

### 3. When to register marketplaces

**Decision:** Register each marketplace immediately after resolving it, before installing any plugins.

**Rationale:** Ensures Claude Code knows about the marketplace before any plugins reference it. Also allows for early failure if marketplace registration fails.

**Alternatives considered:**
- Register after all plugins installed → Risk of partial state if install fails midway
- Register lazily on first plugin → More complex, same result

## Risks / Trade-offs

**[Risk] Cache path changes between skill-manager versions** → Mitigation: Cache path is stable and follows XDG conventions. Any future changes would require migration.

**[Risk] Marketplace not yet cloned when registered** → Mitigation: Registration happens after `ensure_marketplace()` which guarantees the clone exists.

**[Trade-off] Tighter coupling to skill-manager's cache structure** → Acceptable. The `directory` source type is specifically for externally-managed marketplaces, which is exactly our use case.
