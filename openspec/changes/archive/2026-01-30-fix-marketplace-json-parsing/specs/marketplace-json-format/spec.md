## ADDED Requirements

### Requirement: Parse plugins as array
The system SHALL parse the `plugins` field as a JSON array of plugin objects, not a HashMap.

#### Scenario: Parse array of plugins
- **WHEN** marketplace.json contains `"plugins": [{"name": "foo", ...}, {"name": "bar", ...}]`
- **THEN** system parses into a Vec with two MarketplacePlugin entries

### Requirement: Plugin has name field
Each plugin object SHALL have a `name` field containing the plugin identifier.

#### Scenario: Access plugin name
- **WHEN** a plugin object contains `"name": "typescript-lsp"`
- **THEN** `plugin.name` equals `"typescript-lsp"`

### Requirement: Parse local source as string
The system SHALL parse a string `source` field as a local plugin path.

#### Scenario: Local plugin source
- **WHEN** a plugin has `"source": "./plugins/typescript-lsp"`
- **THEN** system parses source as `PluginSource::Local("./plugins/typescript-lsp")`

### Requirement: Parse external source as object
The system SHALL parse an object `source` field with `source: "url"` as an external plugin URL.

#### Scenario: External plugin source
- **WHEN** a plugin has `"source": {"source": "url", "url": "https://github.com/foo/bar.git"}`
- **THEN** system parses source as `PluginSource::External` with url `"https://github.com/foo/bar.git"`

### Requirement: Find plugin by name
The system SHALL provide a method to find a plugin by its name from the parsed plugins list.

#### Scenario: Find existing plugin
- **WHEN** plugins list contains a plugin with `name: "typescript-lsp"`
- **AND** caller requests plugin `"typescript-lsp"`
- **THEN** system returns that plugin

#### Scenario: Plugin not found
- **WHEN** plugins list does not contain a plugin with the requested name
- **THEN** system returns `PluginNotFound` error
