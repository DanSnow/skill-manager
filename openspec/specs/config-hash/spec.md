# config-hash Specification

## Purpose
TBD - created by archiving change fix-lockfile-hash-validation. Update Purpose after archive.
## Requirements
### Requirement: Deterministic manifest hashing

The system SHALL compute a deterministic hash of the manifest content that produces identical results for semantically equivalent configurations.

#### Scenario: Same config produces same hash
- **WHEN** a manifest is parsed and hashed twice
- **THEN** both hashes are identical

#### Scenario: HashMap order does not affect hash
- **WHEN** two manifests have the same marketplaces and plugins in different declaration order
- **THEN** both produce the same hash

#### Scenario: Whitespace and comments do not affect hash
- **WHEN** a manifest has additional whitespace or comments
- **THEN** the hash matches a minimal equivalent manifest

### Requirement: Normalized serialization format

The system SHALL normalize manifests to sorted JSON before hashing to ensure deterministic output.

#### Scenario: Keys are sorted alphabetically
- **WHEN** a manifest is normalized for hashing
- **THEN** marketplace keys and plugin keys are sorted alphabetically

#### Scenario: GitHub shorthand is expanded before hashing
- **WHEN** a manifest contains `official = "anthropics/repo"`
- **THEN** the hash is computed using the expanded URL `https://github.com/anthropics/repo.git`

### Requirement: FxHash algorithm

The system SHALL use FxHash from `rustc-hash` crate for hashing the normalized manifest.

#### Scenario: Hash is 64-bit
- **WHEN** a manifest hash is computed
- **THEN** the result is a u64 value

#### Scenario: Hash is represented as hex string
- **WHEN** a hash is stored in the lock file
- **THEN** it is formatted as a 16-character lowercase hexadecimal string

