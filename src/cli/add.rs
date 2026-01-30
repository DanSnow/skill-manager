use toml_edit::{DocumentMut, Item, Table, Value};

use crate::config::Manifest;
use crate::{Error, Result};

/// Add a plugin to the manifest.
pub fn run(
    name: String,
    marketplace: Option<String>,
    tag: Option<String>,
    commit: Option<String>,
) -> Result<()> {
    // Find the manifest to edit
    let manifest_path = find_manifest()?;

    // Read the manifest file
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| Error::FileRead {
        path: manifest_path.clone(),
        source: e,
    })?;

    // Parse as editable document
    let mut doc: DocumentMut = content
        .parse()
        .map_err(|e: toml_edit::TomlError| Error::ManifestParse(e.to_string()))?;

    // Get the marketplace to use
    let marketplace_name = match marketplace {
        Some(m) => {
            // Verify marketplace exists
            if !marketplace_exists(&doc, &m) {
                return Err(Error::UndeclaredMarketplace(m));
            }
            m
        }
        None => {
            // Search marketplaces for the plugin
            let found = search_marketplaces(&doc, &name)?;
            if found.is_empty() {
                return Err(Error::PluginNotFound {
                    plugin: name,
                    marketplace: "any".to_string(),
                });
            }
            if found.len() == 1 {
                found.into_iter().next().unwrap()
            } else {
                // For now, just use the first one
                // TODO: Interactive selection
                println!("Found in multiple marketplaces: {:?}", found);
                println!("Using first match: {}", found[0]);
                found.into_iter().next().unwrap()
            }
        }
    };

    // Build the plugin entry
    let mut plugin_table = toml_edit::InlineTable::new();
    plugin_table.insert("marketplace", marketplace_name.clone().into());

    if let Some(t) = &tag {
        plugin_table.insert("tag", t.clone().into());
    }
    if let Some(c) = &commit {
        plugin_table.insert("commit", c.clone().into());
    }

    // Ensure [plugins] section exists
    if !doc.contains_table("plugins") {
        doc["plugins"] = Item::Table(Table::new());
    }

    // Add the plugin
    doc["plugins"][&name] = Item::Value(Value::InlineTable(plugin_table));

    // Write back
    std::fs::write(&manifest_path, doc.to_string()).map_err(|e| Error::FileWrite {
        path: manifest_path.clone(),
        source: e,
    })?;

    if let Some(t) = &tag {
        println!("Added {} from {} (tag: {})", name, marketplace_name, t);
    } else if let Some(c) = &commit {
        println!("Added {} from {} (commit: {})", name, marketplace_name, c);
    } else {
        println!("Added {} from {}", name, marketplace_name);
    }

    Ok(())
}

/// Find the manifest to edit (project first, then global).
fn find_manifest() -> Result<std::path::PathBuf> {
    let project_path = Manifest::project_path();
    if project_path.exists() {
        return Ok(project_path);
    }

    if let Some(global_path) = Manifest::global_path() {
        if global_path.exists() {
            return Ok(global_path);
        }
    }

    Err(Error::NoManifest)
}

/// Check if a marketplace is declared in the manifest.
fn marketplace_exists(doc: &DocumentMut, name: &str) -> bool {
    doc.get("marketplaces")
        .and_then(|m| m.as_table())
        .map(|t| t.contains_key(name))
        .unwrap_or(false)
}

/// Search declared marketplaces for a plugin.
/// This is a placeholder - in a real implementation, we'd need to
/// clone/fetch the marketplaces and check their marketplace.json files.
fn search_marketplaces(doc: &DocumentMut, _plugin_name: &str) -> Result<Vec<String>> {
    // For MVP, just return all declared marketplaces
    // The user needs to specify --marketplace or we use the first one
    let marketplaces = doc
        .get("marketplaces")
        .and_then(|m| m.as_table())
        .map(|t| t.iter().map(|(k, _)| k.to_string()).collect())
        .unwrap_or_default();

    Ok(marketplaces)
}
