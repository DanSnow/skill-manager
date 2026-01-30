# marketplace-resolution Specification

## Purpose
TBD - created by archiving change skill-manager-mvp. Update Purpose after archive.
## Requirements
### Requirement: Clone marketplace repository

The system SHALL clone marketplace repositories to the cache directory on first access.

#### Scenario: First-time marketplace clone
- **WHEN** a marketplace has not been cloned
- **THEN** the system clones it to `~/.cache/skill-manager/marketplaces/<name>/`

#### Scenario: Existing marketplace fetch
- **WHEN** a marketplace has been cloned previously
- **THEN** the system fetches updates instead of re-cloning

### Requirement: Resolve marketplace URL formats

The system SHALL expand GitHub shorthand to full HTTPS URLs before cloning.

#### Scenario: Expand GitHub shorthand
- **WHEN** marketplace URL is `owner/repo`
- **THEN** the system clones from `https://github.com/owner/repo.git`

#### Scenario: Use SSH URL directly
- **WHEN** marketplace URL starts with `git@`
- **THEN** the system uses the URL as-is for git clone

#### Scenario: Use HTTPS URL directly
- **WHEN** marketplace URL starts with `https://`
- **THEN** the system uses the URL as-is for git clone

### Requirement: Resolve marketplace version

The system SHALL checkout the appropriate commit based on manifest pinning.

#### Scenario: Resolve tag to commit
- **WHEN** marketplace specifies `tag = "v1.0"`
- **THEN** the system resolves the tag to its commit hash and checks it out

#### Scenario: Use specified commit
- **WHEN** marketplace specifies `commit = "abc123"`
- **THEN** the system checks out that exact commit

#### Scenario: Use HEAD for unpinned
- **WHEN** marketplace has no tag or commit
- **THEN** the system uses HEAD of the default branch

### Requirement: Parse marketplace.json

The system SHALL parse `marketplace.json` from cloned marketplaces to discover available plugins.

#### Scenario: Read plugin list
- **WHEN** marketplace is cloned
- **THEN** the system reads `marketplace.json` to get the list of available plugins

#### Scenario: Missing marketplace.json
- **WHEN** marketplace has no `marketplace.json`
- **THEN** the system returns an error indicating invalid marketplace format

### Requirement: Resolve plugin source type

The system SHALL determine if a plugin is local (in marketplace repo) or external (separate repo).

#### Scenario: Local plugin resolution
- **WHEN** plugin entry in marketplace.json has a `path` field
- **THEN** the system marks it as `source_type = "local"` and uses the path within the marketplace

#### Scenario: External plugin resolution
- **WHEN** plugin entry in marketplace.json has a `url` field
- **THEN** the system marks it as `source_type = "external"` and clones that URL

### Requirement: Resolve plugin version

The system SHALL resolve plugin versions independent of marketplace versions.

#### Scenario: Plugin tag resolution
- **WHEN** plugin specifies `tag = "v4.1.1"` and is external
- **THEN** the system resolves the tag from the plugin's repository

#### Scenario: Plugin commit resolution
- **WHEN** plugin specifies `commit = "def456"` and is external
- **THEN** the system uses that commit from the plugin's repository

#### Scenario: Plugin inherits marketplace commit for local
- **WHEN** plugin is local and has no version pin
- **THEN** the plugin version is tied to the marketplace commit

### Requirement: Handle git errors

The system SHALL provide clear error messages for git operation failures.

#### Scenario: Clone failure
- **WHEN** git clone fails (network error, auth failure)
- **THEN** the system returns an error with the marketplace name and underlying git error

#### Scenario: Tag not found
- **WHEN** specified tag does not exist
- **THEN** the system returns an error listing available tags

#### Scenario: Commit not found
- **WHEN** specified commit does not exist
- **THEN** the system returns an error indicating the commit hash is invalid

