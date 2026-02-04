## 1. Dependencies

- [x] 1.1 Add `rustc-hash` crate to Cargo.toml

## 2. Manifest Hashing

- [x] 2.1 Create `NormalizedManifest` struct with `BTreeMap` fields in `src/config/manifest.rs`
- [x] 2.2 Implement `Manifest::to_normalized()` method to convert HashMap to sorted BTreeMap
- [x] 2.3 Implement `Manifest::compute_hash()` using serde_json serialization and FxHasher
- [x] 2.4 Add unit tests for hash determinism (same config = same hash, order-independent)

## 3. Lock File Format

- [x] 3.1 Add `config_hash: Option<String>` field to `LockFile` struct in `src/config/lockfile.rs`
- [x] 3.2 Update `LockFile::to_string()` to include `config_hash` field in TOML output
- [x] 3.3 Add unit test for lock file serialization with config_hash

## 4. Install Command

- [x] 4.1 Update `src/cli/install.rs` to compute manifest hash before checking lock file
- [x] 4.2 Add hash comparison logic: re-resolve if hash missing or mismatched
- [x] 4.3 Pass computed hash to `LockFile` when saving
- [x] 4.4 Add integration test: config change triggers re-resolution

## 5. Verification

- [x] 5.1 Test backward compatibility: old lock file without hash triggers re-resolution
- [x] 5.2 Test happy path: unchanged config uses locked versions
- [x] 5.3 Test config change detection: adding/removing plugin triggers re-resolution
