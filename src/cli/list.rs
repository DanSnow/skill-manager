use crate::config::{LockFile, Manifest};
use crate::Result;

/// List plugins from the manifest.
pub fn run() -> Result<()> {
    // Load manifests
    let global_manifest = Manifest::load_global()?;
    let project_manifest = Manifest::load_project()?;

    if global_manifest.is_none() && project_manifest.is_none() {
        println!("No plugins.toml found. Run `skill-manager init` to create one.");
        return Ok(());
    }

    // Display plugins from each manifest
    if let Some(ref manifest) = project_manifest {
        let manifest_path = manifest.path.as_ref().unwrap();
        let lock_path = LockFile::path_for_manifest(manifest_path);
        let lock = LockFile::load_if_exists(&lock_path)?;

        println!("Project plugins ({}):", manifest_path.display());
        if manifest.plugins.is_empty() {
            println!("  (none)");
        } else {
            list_plugins(manifest, lock.as_ref())?;
        }
        println!();
    }

    if let Some(ref manifest) = global_manifest {
        let manifest_path = manifest.path.as_ref().unwrap();
        let lock_path = LockFile::path_for_manifest(manifest_path);
        let lock = LockFile::load_if_exists(&lock_path)?;

        println!("Global plugins ({}):", manifest_path.display());
        if manifest.plugins.is_empty() {
            println!("  (none)");
        } else {
            list_plugins(manifest, lock.as_ref())?;
        }
    }

    Ok(())
}

fn list_plugins(manifest: &Manifest, lock: Option<&LockFile>) -> Result<()> {
    for (name, plugin) in &manifest.plugins {
        let mut parts = vec![format!("  {} ({})", name, plugin.marketplace)];

        // Show version from manifest if specified
        if let Some(ref tag) = plugin.tag {
            parts.push(format!("tag: {}", tag));
        } else if let Some(ref commit) = plugin.commit {
            parts.push(format!("commit: {}", &commit[..7.min(commit.len())]));
        }

        // Show lock status
        if let Some(ref lock) = lock {
            if let Some(pkg) = lock.find_package(name) {
                parts.push(format!("v{}", pkg.resolved_version));
                parts.push(format!("[locked: {}]", &pkg.plugin_commit[..7.min(pkg.plugin_commit.len())]));
            } else {
                parts.push("[not locked]".to_string());
            }
        } else {
            parts.push("[no lock file]".to_string());
        }

        println!("{}", parts.join(" "));
    }

    Ok(())
}
