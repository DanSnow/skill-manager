# marketplace-registration Specification

## Purpose
TBD - created by archiving change write-marketplace-info. Update Purpose after archive.
## Requirements
### Requirement: Register marketplace in known_marketplaces.json

The system SHALL write marketplace entries to `~/.claude/plugins/known_marketplaces.json` when installing plugins from a marketplace.

Each entry SHALL contain:
- Key: marketplace name (e.g., `"official"`)
- `source`: Object with `"source": "directory"` and `"path"` pointing to skill-manager's marketplace cache
- `installLocation`: Same path as `source.path`
- `lastUpdated`: ISO 8601 timestamp

#### Scenario: Register marketplace with directory source
- **WHEN** installing plugins from a marketplace named `"official"`
- **THEN** the system writes an entry to `~/.claude/plugins/known_marketplaces.json` with:
  - `source.source` = `"directory"`
  - `source.path` = `"~/.cache/skill-manager/marketplaces/official"` (expanded to absolute path)
  - `installLocation` = same as `source.path`
  - `lastUpdated` = current ISO 8601 timestamp

#### Scenario: Claude Code does not update marketplace
- **WHEN** Claude Code reads a marketplace entry with `source.source` = `"directory"`
- **THEN** Claude Code uses the local directory without attempting to fetch or update it

### Requirement: Preserve existing marketplace entries

The system SHALL preserve existing entries in `known_marketplaces.json` when adding new marketplaces, regardless of their source type.

#### Scenario: Add marketplace to existing file
- **WHEN** `known_marketplaces.json` already contains entries for other marketplaces
- **THEN** the system preserves all existing entries and adds the new marketplace entry

#### Scenario: Update existing marketplace entry
- **WHEN** `known_marketplaces.json` already contains an entry for the same marketplace name
- **THEN** the system updates that entry with new `lastUpdated` timestamp and current path

#### Scenario: Preserve entries with github source type
- **WHEN** `known_marketplaces.json` contains an entry with `source.source` = `"github"`
- **THEN** the system preserves that entry exactly as-is (does not modify or fail)

#### Scenario: Preserve entries with unknown source types
- **WHEN** `known_marketplaces.json` contains an entry with an unknown source type
- **THEN** the system preserves that entry exactly as-is (does not modify or fail)

### Requirement: Create known_marketplaces.json if not exists

The system SHALL create `~/.claude/plugins/known_marketplaces.json` if the file does not exist.

#### Scenario: First marketplace registration
- **WHEN** `~/.claude/plugins/known_marketplaces.json` does not exist
- **THEN** the system creates the file with the marketplace entry

#### Scenario: Parent directory does not exist
- **WHEN** `~/.claude/plugins/` directory does not exist
- **THEN** the system creates the directory and the `known_marketplaces.json` file

### Requirement: Register marketplaces before plugin installation

The system SHALL register each marketplace in `known_marketplaces.json` before installing any plugins from that marketplace.

#### Scenario: Installation order
- **WHEN** running `skill-manager install`
- **THEN** each marketplace is registered in `known_marketplaces.json` after being cloned/fetched but before its plugins are extracted and registered in `installed_plugins.json`

