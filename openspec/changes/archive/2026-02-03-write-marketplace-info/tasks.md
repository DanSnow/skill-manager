## 1. Data Structures

- [x] 1.1 Add `KnownMarketplaceEntry` struct with `source: serde_json::Value`, `install_location: String`, and `last_updated: String` fields in `src/installer/claude.rs`
- [x] 1.2 Add helper function `make_directory_source(path: &Path) -> serde_json::Value` that creates `{"source": "directory", "path": "..."}` JSON value

## 2. File Operations

- [x] 2.1 Add `known_marketplaces_path()` method returning `~/.claude/plugins/known_marketplaces.json`
- [x] 2.2 Add `read_known_marketplaces()` method to read existing file as `HashMap<String, KnownMarketplaceEntry>` or return empty HashMap
- [x] 2.3 Add `write_known_marketplaces()` method to write file with pretty JSON formatting
- [x] 2.4 Add `register_marketplace()` method that creates entry with directory source and upserts into file

## 3. Integration

- [x] 3.1 Call `register_marketplace()` in `src/cli/install.rs` after `ensure_marketplace()` succeeds, before plugin installation loop
- [x] 3.2 Pass marketplace name and cache path from `MarketplaceResolver::marketplace_path()` to registration

## 4. Testing

- [x] 4.1 Add unit tests for `read_known_marketplaces()` (empty file, existing entries with various source types)
- [x] 4.2 Add unit tests for `register_marketplace()` (new entry, update existing, preserve entries with github/url source types)
