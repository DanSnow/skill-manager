# conflict-handling Specification

## Purpose
TBD - created by archiving change skill-manager-mvp. Update Purpose after archive.
## Requirements
### Requirement: Detect global-project conflicts

The system SHALL detect when a plugin exists in both global and project manifests at different versions.

#### Scenario: Same plugin different versions
- **WHEN** global manifest has `superpowers` at v4.0.0 and project has it at v4.1.1
- **THEN** the system detects this as a conflict

#### Scenario: Same plugin same version
- **WHEN** both manifests have the same plugin at the same version
- **THEN** there is no conflict

#### Scenario: Plugin only in one manifest
- **WHEN** a plugin exists only in global or only in project
- **THEN** there is no conflict

### Requirement: Display conflict information

The system SHALL display clear information about detected conflicts.

#### Scenario: Conflict message format
- **WHEN** a conflict is detected during install
- **THEN** the system displays the plugin name, global version, and project version

#### Scenario: Explain Claude Code behavior
- **WHEN** displaying a conflict
- **THEN** the system notes that "Claude Code loads global over project plugins"

### Requirement: Offer conflict resolution options

The system SHALL prompt the user to resolve conflicts interactively.

#### Scenario: Present resolution options
- **WHEN** a conflict is detected
- **THEN** the system offers: (1) Update global to project version, (2) Skip this plugin, (3) Abort installation

#### Scenario: Update global option
- **WHEN** user selects "Update global"
- **THEN** the system updates the global manifest to match project version and continues

#### Scenario: Skip option
- **WHEN** user selects "Skip"
- **THEN** the system keeps the global version and skips the project plugin

#### Scenario: Abort option
- **WHEN** user selects "Abort"
- **THEN** the system exits without making changes

### Requirement: Non-interactive conflict handling

The system SHALL support non-interactive conflict resolution via flags.

#### Scenario: Force global flag
- **WHEN** user runs `skill-manager install --prefer-global`
- **THEN** conflicts are resolved by keeping global versions without prompting

#### Scenario: Force project flag
- **WHEN** user runs `skill-manager install --prefer-project`
- **THEN** conflicts are resolved by updating to project versions without prompting

### Requirement: Warn about manual resolution

The system SHALL inform users when manual steps may be required.

#### Scenario: Note about Claude Code restart
- **WHEN** conflict is resolved by updating global
- **THEN** the system notes that Claude Code may need restart to pick up changes

#### Scenario: Note about project limitations
- **WHEN** user skips a project plugin due to conflict
- **THEN** the system notes that the project-specified version will not be used

