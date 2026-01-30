## ADDED Requirements

### Requirement: init command

The system SHALL provide an `init` command that creates a new `plugins.toml` manifest.

#### Scenario: Project init
- **WHEN** user runs `skill-manager init`
- **THEN** the system creates `./.claude/plugins.toml` with empty sections

#### Scenario: Global init
- **WHEN** user runs `skill-manager init --global`
- **THEN** the system creates `~/.config/skill-manager/plugins.toml`

#### Scenario: Init with existing manifest
- **WHEN** user runs `skill-manager init` and manifest already exists
- **THEN** the system exits with an error message indicating the file exists

### Requirement: add command

The system SHALL provide an `add` command that adds a plugin entry to the manifest.

#### Scenario: Add plugin with marketplace
- **WHEN** user runs `skill-manager add typescript-lsp --marketplace official`
- **THEN** the system adds `typescript-lsp = { marketplace = "official" }` to `[plugins]`

#### Scenario: Add plugin with tag
- **WHEN** user runs `skill-manager add superpowers --marketplace superpowers --tag v4.1.1`
- **THEN** the system adds `superpowers = { marketplace = "superpowers", tag = "v4.1.1" }` to `[plugins]`

#### Scenario: Add plugin with commit
- **WHEN** user runs `skill-manager add sourceatlas --marketplace sourceatlas --commit def456`
- **THEN** the system adds `sourceatlas = { marketplace = "sourceatlas", commit = "def456" }` to `[plugins]`

#### Scenario: Add plugin without marketplace searches
- **WHEN** user runs `skill-manager add typescript-lsp` without `--marketplace`
- **THEN** the system searches known marketplaces and prompts user to choose

#### Scenario: Add plugin with unknown marketplace
- **WHEN** user specifies a marketplace not in the manifest
- **THEN** the system prompts to add the marketplace first

### Requirement: install command

The system SHALL provide an `install` command that installs plugins from the manifest.

#### Scenario: Install with existing lock file
- **WHEN** user runs `skill-manager install` and `plugins.lock` exists
- **THEN** the system installs exact versions from the lock file

#### Scenario: Install without lock file
- **WHEN** user runs `skill-manager install` and no `plugins.lock` exists
- **THEN** the system resolves versions, creates lock file, then installs

#### Scenario: Install with update flag
- **WHEN** user runs `skill-manager install --update`
- **THEN** the system re-resolves all versions and updates the lock file

#### Scenario: Install creates cache directories
- **WHEN** installation runs
- **THEN** the system creates `~/.cache/skill-manager/marketplaces/` and `~/.cache/skill-manager/plugins/`

#### Scenario: Install updates Claude Code files
- **WHEN** installation completes
- **THEN** the system writes entries to `~/.claude/plugins/installed_plugins.json` and `~/.claude/settings.json`

### Requirement: remove command

The system SHALL provide a `remove` command that removes a plugin from the manifest.

#### Scenario: Remove existing plugin
- **WHEN** user runs `skill-manager remove typescript-lsp`
- **THEN** the system removes the `typescript-lsp` entry from `[plugins]`

#### Scenario: Remove non-existent plugin
- **WHEN** user runs `skill-manager remove nonexistent`
- **THEN** the system exits with an error message

#### Scenario: Remove does not uninstall
- **WHEN** user runs `skill-manager remove typescript-lsp`
- **THEN** the system does NOT modify Claude Code's JSON files (manual uninstall required)

### Requirement: list command

The system SHALL provide a `list` command that shows installed plugins.

#### Scenario: List shows plugin details
- **WHEN** user runs `skill-manager list`
- **THEN** the system displays each plugin with name, marketplace, version, and lock status

#### Scenario: List indicates version drift
- **WHEN** a running plugin version differs from the locked version
- **THEN** the system indicates the drift in the output

#### Scenario: List with empty manifest
- **WHEN** user runs `skill-manager list` with no plugins declared
- **THEN** the system displays a message indicating no plugins are configured

### Requirement: Command help

The system SHALL provide `--help` for all commands showing usage and options.

#### Scenario: Root help
- **WHEN** user runs `skill-manager --help`
- **THEN** the system displays available commands and global options

#### Scenario: Command-specific help
- **WHEN** user runs `skill-manager install --help`
- **THEN** the system displays install command options and usage
