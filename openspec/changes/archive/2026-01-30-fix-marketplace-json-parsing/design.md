## Context

The skill-manager needs to parse marketplace.json files from the official Claude plugins repository. The current implementation uses a custom format that doesn't match the official schema, causing parse failures.

Current state:
- `MarketplaceJson.plugins` is `HashMap<String, MarketplacePlugin>`
- Plugin name is the HashMap key
- Uses `path: Option<String>` and `url: Option<String>` for source location

Official format:
- `plugins` is an array `Vec<MarketplacePlugin>`
- Each plugin has a `name` field
- Uses polymorphic `source` field (string or object)

## Goals / Non-Goals

**Goals:**
- Parse official Claude plugins marketplace.json format
- Handle polymorphic `source` field (local string vs external object)
- Maintain ability to find plugins by name efficiently

**Non-Goals:**
- Supporting additional marketplace.json fields (category, homepage, tags, etc.) - these are optional
- Backward compatibility with the old custom format
- Validating against the official JSON schema

## Decisions

### 1. Use `#[serde(untagged)]` for PluginSource enum

**Decision**: Define `PluginSource` as an untagged enum to handle the polymorphic source field.

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PluginSource {
    Local(String),
    External { source: String, url: String },
}
```

**Rationale**: The official format uses:
- `"source": "./plugins/foo"` for local
- `"source": {"source": "url", "url": "..."}` for external

Untagged enums try each variant in order. Local (simple string) will match first, External (object) will match for the complex case.

**Alternative considered**: Custom deserializer - more complex, no benefit for this case.

### 2. Keep plugins as Vec, build HashMap on demand

**Decision**: Store as `Vec<MarketplacePlugin>` matching the JSON, create lookup HashMap when needed in `find_plugin`.

**Rationale**:
- Direct mapping from JSON = simpler deserialization
- Plugin lookup is infrequent (once per install)
- Avoids pre-processing step

**Alternative considered**: Deserialize to HashMap via `#[serde(deserialize_with)]` - adds complexity, marginal benefit.

## Risks / Trade-offs

- **[O(n) lookup]** → Acceptable for small plugin counts (<100). Could cache HashMap if needed later.
- **[Untagged enum order matters]** → Local variant must come first (simpler type matched before complex).
