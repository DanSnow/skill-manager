## MODIFIED Requirements

### Requirement: Write to installed_plugins.json

The system SHALL add entries to Claude Code's `~/.claude/plugins/installed_plugins.json` using the v2 format with wrapper object and arrays.

#### Scenario: Parse v2 format
- **WHEN** reading installed_plugins.json
- **THEN** the system parses the wrapper object with `version` and `plugins` fields

#### Scenario: Handle array-per-plugin
- **WHEN** reading a plugin entry
- **THEN** the system reads an array of installation entries (supporting multiple scopes)

#### Scenario: Add user-scope plugin entry
- **WHEN** a plugin is installed from the global manifest
- **THEN** the system adds an entry with `scope: "user"`, `installPath`, `version`, `installedAt`, `lastUpdated`, and `gitCommitSha`

#### Scenario: Add project-scope plugin entry
- **WHEN** a plugin is installed from a project manifest
- **THEN** the system adds an entry with `scope: "project"`, `projectPath` (canonicalized absolute path), `installPath`, `version`, `installedAt`, `lastUpdated`, and `gitCommitSha`

#### Scenario: Plugin key format
- **WHEN** writing to installed_plugins.json
- **THEN** the key is formatted as `<plugin>@<marketplace>`

#### Scenario: Preserve entries with different scopes
- **WHEN** updating a user-scope entry and project-scope entries exist for the same plugin
- **THEN** the system preserves all project-scope entries

#### Scenario: Preserve entries with different project paths
- **WHEN** updating a project-scope entry for project A
- **THEN** the system preserves project-scope entries for other projects

#### Scenario: Replace existing entry with same scope
- **WHEN** a user-scope entry exists and installing user-scope
- **THEN** the system replaces the existing user-scope entry

#### Scenario: Replace existing project entry with same path
- **WHEN** a project-scope entry exists for the same project path
- **THEN** the system replaces that specific project-scope entry

#### Scenario: Create file if missing
- **WHEN** installed_plugins.json does not exist
- **THEN** the system creates it with version 2 and the new entry

#### Scenario: Write v2 format
- **WHEN** writing installed_plugins.json
- **THEN** the system writes the wrapper object with `version: 2` and `plugins` containing arrays
