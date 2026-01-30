## ADDED Requirements

### Requirement: Extract plugin to cache

The system SHALL extract resolved plugins to the cache directory with commit-based paths.

#### Scenario: Local plugin extraction
- **WHEN** a local plugin is resolved
- **THEN** the system copies it to `~/.cache/skill-manager/plugins/<marketplace>/<plugin>/<commit>/`

#### Scenario: External plugin extraction
- **WHEN** an external plugin is resolved
- **THEN** the system clones and checks out to `~/.cache/skill-manager/plugins/<marketplace>/<plugin>/<commit>/`

#### Scenario: Skip existing extraction
- **WHEN** the target path already exists with correct commit
- **THEN** the system skips extraction (cached)

### Requirement: Read plugin.json for metadata

The system SHALL read `plugin.json` from extracted plugins for version and metadata.

#### Scenario: Extract version from plugin.json
- **WHEN** plugin is extracted
- **THEN** the system reads `version` field from `plugin.json` for display purposes

#### Scenario: Missing plugin.json
- **WHEN** extracted plugin has no `plugin.json`
- **THEN** the system uses "unknown" as resolved_version

### Requirement: Write to installed_plugins.json

The system SHALL add entries to Claude Code's `~/.claude/plugins/installed_plugins.json`.

#### Scenario: Add plugin entry
- **WHEN** a plugin is installed
- **THEN** the system adds an entry with `scope`, `installPath`, `version`, `installedAt`, `lastUpdated`, and `gitCommitSha`

#### Scenario: Plugin key format
- **WHEN** writing to installed_plugins.json
- **THEN** the key is formatted as `<plugin>@<marketplace>`

#### Scenario: Preserve existing entries
- **WHEN** installed_plugins.json has existing entries
- **THEN** the system preserves entries not managed by skill-manager

#### Scenario: Create file if missing
- **WHEN** installed_plugins.json does not exist
- **THEN** the system creates it with the new entry

### Requirement: Write to settings.json

The system SHALL enable installed plugins in Claude Code's `~/.claude/settings.json`.

#### Scenario: Enable plugin in settings
- **WHEN** a plugin is installed
- **THEN** the system sets `enabledPlugins["<plugin>@<marketplace>"] = true`

#### Scenario: Preserve existing settings
- **WHEN** settings.json has other configuration
- **THEN** the system preserves all existing keys

#### Scenario: Create enabledPlugins if missing
- **WHEN** settings.json exists but has no `enabledPlugins`
- **THEN** the system adds the `enabledPlugins` object

### Requirement: Create CACHEDIR.TAG

The system SHALL create a CACHEDIR.TAG file in the cache directory.

#### Scenario: Create cache tag
- **WHEN** cache directory is created
- **THEN** the system writes `~/.cache/skill-manager/CACHEDIR.TAG` with standard signature

#### Scenario: Tag content format
- **WHEN** CACHEDIR.TAG is created
- **THEN** it starts with `Signature: 8a477f597d28d172789f06886806bc55`

### Requirement: Installation progress output

The system SHALL display progress during installation.

#### Scenario: Show marketplace cloning
- **WHEN** cloning a marketplace
- **THEN** the system displays "Cloning marketplace '<name>'..."

#### Scenario: Show plugin installation
- **WHEN** installing a plugin
- **THEN** the system displays "Installing <name>..."

#### Scenario: Show completion summary
- **WHEN** installation completes
- **THEN** the system displays count of installed plugins
