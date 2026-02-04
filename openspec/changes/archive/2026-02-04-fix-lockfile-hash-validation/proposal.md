## Why

The lock file is never updated after initial creation, even when `plugins.toml` changes. The current logic at `src/cli/install.rs:55-59` only checks if the lock file exists - if it does, it uses the locked versions unconditionally. This means adding, removing, or modifying plugins in `plugins.toml` has no effect until the user manually runs `install --update` or deletes the lock file.

## What Changes

- Add a deterministic content hash to `plugins.lock` that captures the normalized state of `plugins.toml`
- On install, compute the current config hash and compare against the stored hash
- If hashes differ (config changed), treat it as if no lock file exists and re-resolve
- If hashes match (config unchanged), use locked versions as before
- The hash uses sorted keys and stable serialization to ensure deterministic output

## Capabilities

### New Capabilities

- `config-hash`: Deterministic hashing of plugin configuration for change detection

### Modified Capabilities

- `manifest-format`: Add `config_hash` field to `plugins.lock` structure for storing the normalized config hash

## Impact

- **Lock file format**: New `config_hash` field added to `plugins.lock` header (backward compatible - missing hash triggers re-resolution)
- **Install behavior**: Install now automatically detects config changes instead of requiring `--update`
- **Files affected**: `src/config/lockfile.rs`, `src/cli/install.rs`, potentially `src/config/manifest.rs` for hash computation
