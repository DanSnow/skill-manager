## MODIFIED Requirements

### Requirement: plugins.lock structure

The system SHALL generate a `plugins.lock` file containing a `config_hash` field, `[[marketplace]]` and `[[package]]` arrays with resolved commit hashes.

#### Scenario: Lock file contains config hash
- **WHEN** a lock file is generated
- **THEN** the top-level `config_hash` field contains the manifest hash

#### Scenario: Lock file contains marketplace commits
- **WHEN** installation completes
- **THEN** each `[[marketplace]]` entry includes `name`, `url`, and `commit` fields

#### Scenario: Lock file contains package resolution
- **WHEN** installation completes
- **THEN** each `[[package]]` entry includes `name`, `marketplace`, `source_type`, `marketplace_commit`, and `resolved_version` fields

#### Scenario: Missing config_hash triggers re-resolution
- **WHEN** a lock file exists but has no `config_hash` field
- **THEN** the system re-resolves all plugins (backward compatibility)

#### Scenario: Mismatched config_hash triggers re-resolution
- **WHEN** a lock file exists but `config_hash` differs from current manifest hash
- **THEN** the system re-resolves all plugins

#### Scenario: Matching config_hash uses locked versions
- **WHEN** a lock file exists and `config_hash` matches current manifest hash
- **THEN** the system uses the locked versions without re-resolving
