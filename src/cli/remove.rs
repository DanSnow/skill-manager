use toml_edit::DocumentMut;

use crate::config::Manifest;
use crate::{Error, Result};

/// Remove a plugin from the manifest.
pub fn run(name: String) -> Result<()> {
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

    // Check if plugin exists
    let plugins = doc
        .get_mut("plugins")
        .and_then(|p| p.as_table_mut());

    match plugins {
        Some(table) if table.contains_key(&name) => {
            table.remove(&name);
        }
        _ => {
            return Err(Error::PluginNotInManifest(name));
        }
    }

    // Write back
    std::fs::write(&manifest_path, doc.to_string()).map_err(|e| Error::FileWrite {
        path: manifest_path.clone(),
        source: e,
    })?;

    println!("Removed {} from {}", name, manifest_path.display());
    println!("Note: The plugin is still installed. Run `skill-manager install` to sync.");

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
