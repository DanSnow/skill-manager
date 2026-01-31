## 1. Create PluginLayout Module

- [x] 1.1 Create `src/layout.rs` with `PluginLayout` struct using `OnceCell` for lazy path caching
- [x] 1.2 Implement `new()`, `base_path()`, `config_dir()`, `plugin_json()`, `marketplace_json()` methods
- [x] 1.3 Add `pub mod layout;` to `src/lib.rs` and export `PluginLayout`
- [x] 1.4 Add unit tests for `PluginLayout` path methods

## 2. Refactor ResolvedPlugin Construction

- [x] 2.1 Add `from_local()` constructor to `ResolvedPlugin` in `src/resolver/plugin.rs`
- [x] 2.2 Add `from_external()` constructor to `ResolvedPlugin`
- [x] 2.3 Add private `read_version(layout: &PluginLayout) -> Option<String>` method
- [x] 2.4 Implement SHA fallback logic (first 7 chars of commit) in constructors

## 3. Update MarketplaceResolver

- [x] 3.1 Update `resolve_local_plugin()` to use `PluginLayout` and `ResolvedPlugin::from_local()`
- [x] 3.2 Update `resolve_external_plugin()` to use `PluginLayout` and `ResolvedPlugin::from_external()`
- [x] 3.3 Remove old `read_plugin_version()` method from `MarketplaceResolver`

## 4. Testing

- [x] 4.1 Update existing `test_resolve_local_plugin` to verify correct version from `.claude-plugin/plugin.json`
- [x] 4.2 Add test for SHA fallback when plugin.json is missing
- [x] 4.3 Add test for SHA fallback when version field is missing from plugin.json
- [x] 4.4 Run full test suite and fix any regressions
