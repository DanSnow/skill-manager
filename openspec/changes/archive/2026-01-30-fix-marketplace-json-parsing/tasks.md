## 1. Update Data Structures

- [x] 1.1 Add `PluginSource` enum with `#[serde(untagged)]` for Local(String) and External { source, url }
- [x] 1.2 Update `MarketplacePlugin` struct: add `name: String`, replace `path`/`url` with `source: PluginSource`
- [x] 1.3 Update `MarketplaceJson` struct: change `plugins` from `HashMap<String, MarketplacePlugin>` to `Vec<MarketplacePlugin>`

## 2. Update find_plugin Method

- [x] 2.1 Update `find_plugin` to iterate Vec and match by `plugin.name` instead of HashMap lookup

## 3. Update Tests

- [x] 3.1 Update `setup_test_repo` to use new JSON format (array with name/source fields)
- [x] 3.2 Update test assertions to use new struct fields
