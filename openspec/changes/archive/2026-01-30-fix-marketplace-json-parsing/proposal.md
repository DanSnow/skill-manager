## Why

The current `MarketplaceJson` struct expects `plugins` to be a `HashMap<String, MarketplacePlugin>` where the plugin name is the key. The official Claude plugins marketplace uses an array format where each plugin has a `name` field inside the object, and uses `source` instead of `path`/`url` fields with polymorphic typing (string for local, object for external).

## What Changes

- Change `plugins` field from `HashMap<String, MarketplacePlugin>` to `Vec<MarketplacePlugin>`
- Add `name` field to `MarketplacePlugin` struct
- Replace `path` and `url` fields with a polymorphic `source` field that can be:
  - A string (local path like `"./plugins/foo"`)
  - An object with `source: "url"` and `url` fields for external plugins
- Add optional `description` field (already exists, keep it)

## Capabilities

### New Capabilities

- `marketplace-json-format`: Define the expected structure of marketplace.json files matching the official Claude plugins format

### Modified Capabilities

(none - no existing specs)

## Impact

- `src/resolver/marketplace.rs`: Update `MarketplaceJson` and `MarketplacePlugin` structs
- `src/resolver/marketplace.rs`: Update `find_plugin` to search by name in the Vec
- Tests in `marketplace.rs` need updating to use new format
- Any code that accesses plugin fields via the old `path`/`url` fields needs to use the new `source` enum
