## ADDED Requirements

### Requirement: PluginLayout provides base path access
The `PluginLayout` struct SHALL store a base path and provide access via `base_path()` method returning `&Path`.

#### Scenario: Access base path
- **WHEN** `PluginLayout::new("/path/to/plugin")` is called
- **THEN** `layout.base_path()` SHALL return a reference to `/path/to/plugin`

### Requirement: PluginLayout provides config directory path
The `PluginLayout` struct SHALL provide a `config_dir()` method that returns `&Path` to the `.claude-plugin` directory, lazily computed via `OnceCell`.

#### Scenario: Access config directory
- **WHEN** `layout.config_dir()` is called
- **THEN** it SHALL return a reference to `<base_path>/.claude-plugin`

#### Scenario: Config directory is lazily cached
- **WHEN** `layout.config_dir()` is called multiple times
- **THEN** the path SHALL be computed once and cached via `OnceCell`

### Requirement: PluginLayout provides plugin.json path
The `PluginLayout` struct SHALL provide a `plugin_json()` method that returns `&Path` to the plugin.json file location, lazily computed via `OnceCell`.

#### Scenario: Access plugin.json path
- **WHEN** `layout.plugin_json()` is called
- **THEN** it SHALL return a reference to `<base_path>/.claude-plugin/plugin.json`

### Requirement: PluginLayout provides marketplace.json path
The `PluginLayout` struct SHALL provide a `marketplace_json()` method that returns `&Path` to the marketplace.json file location, lazily computed via `OnceCell`.

#### Scenario: Access marketplace.json path
- **WHEN** `layout.marketplace_json()` is called
- **THEN** it SHALL return a reference to `<base_path>/.claude-plugin/marketplace.json`

### Requirement: ResolvedPlugin reads version from correct location
The `ResolvedPlugin` struct SHALL read version from `plugin.json` at the path provided by `PluginLayout::plugin_json()`.

#### Scenario: Version found in plugin.json
- **WHEN** `ResolvedPlugin::from_local(...)` or `ResolvedPlugin::from_external(...)` is called
- **AND** `<base_path>/.claude-plugin/plugin.json` exists with a `version` field
- **THEN** `resolved_version` SHALL be set to that version value

### Requirement: ResolvedPlugin falls back to git SHA when version unavailable
When `plugin.json` does not exist or has no version field, `ResolvedPlugin` SHALL use the first 7 characters of the commit SHA as `resolved_version`.

#### Scenario: Version missing - use SHA fallback
- **WHEN** `ResolvedPlugin::from_local(...)` is called
- **AND** `plugin.json` does not exist or has no `version` field
- **THEN** `resolved_version` SHALL be set to the first 7 characters of the commit SHA

#### Scenario: External plugin version missing - use plugin commit SHA
- **WHEN** `ResolvedPlugin::from_external(...)` is called
- **AND** `plugin.json` does not exist or has no `version` field
- **THEN** `resolved_version` SHALL be set to the first 7 characters of `plugin_commit`

### Requirement: ResolvedPlugin owns construction logic
The `ResolvedPlugin` struct SHALL provide `from_local()` and `from_external()` constructor methods that handle version resolution internally.

#### Scenario: Construct local plugin
- **WHEN** `ResolvedPlugin::from_local(name, marketplace, commit, source, layout)` is called
- **THEN** a fully populated `ResolvedPlugin` SHALL be returned with `source_type: Local`

#### Scenario: Construct external plugin
- **WHEN** `ResolvedPlugin::from_external(name, marketplace, marketplace_commit, plugin_commit, source, layout)` is called
- **THEN** a fully populated `ResolvedPlugin` SHALL be returned with `source_type: External`
