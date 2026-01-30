use crate::config::{LockFile, LockedMarketplace, LockedPackage, Manifest, SourceType};
use crate::installer::{CacheManager, ClaudeCodeIntegration};
use crate::resolver::MarketplaceResolver;
use crate::{Error, Result};

/// Install plugins from the manifest.
pub fn run(update: bool, _prefer_global: bool, _prefer_project: bool) -> Result<()> {
    // Load manifests
    let global_manifest = Manifest::load_global()?;
    let project_manifest = Manifest::load_project()?;

    if global_manifest.is_none() && project_manifest.is_none() {
        return Err(Error::NoManifest);
    }

    // For MVP, we'll just handle whichever manifest exists
    // TODO: Merge manifests and handle conflicts
    let manifest = project_manifest
        .or(global_manifest)
        .ok_or(Error::NoManifest)?;

    let manifest_path = manifest.path.clone().unwrap();
    manifest.validate()?;

    // Initialize components
    let cache = CacheManager::new()?;
    cache.ensure_cache_dir()?;

    let resolver = MarketplaceResolver::new(cache.cache_dir().to_path_buf());
    let claude = ClaudeCodeIntegration::new();

    // Check for existing lock file
    let lock_path = LockFile::path_for_manifest(&manifest_path);
    let existing_lock = if !update {
        LockFile::load_if_exists(&lock_path)?
    } else {
        None
    };

    // Resolve or use locked versions
    let (locked_marketplaces, locked_packages) = if let Some(ref lock) = existing_lock {
        println!("Using locked versions from {}", lock_path.display());
        (lock.marketplaces.clone(), lock.packages.clone())
    } else {
        println!("Resolving plugin versions...");
        resolve_all(&manifest, &resolver)?
    };

    // Create/update lock file
    let lock_file = LockFile {
        marketplaces: locked_marketplaces.clone(),
        packages: locked_packages.clone(),
        path: Some(lock_path.clone()),
    };

    if existing_lock.is_none() || update {
        lock_file.save(&lock_path)?;
        println!("Wrote {}", lock_path.display());
    }

    // Install plugins
    let mut installed_count = 0;
    for pkg in &locked_packages {
        let marketplace = locked_marketplaces
            .iter()
            .find(|m| m.name == pkg.marketplace)
            .ok_or_else(|| Error::UndeclaredMarketplace(pkg.marketplace.clone()))?;

        println!("Installing {}...", pkg.name);

        // Extract plugin to cache
        let install_path = match pkg.source_type {
            SourceType::Local => {
                let marketplace_path = resolver.marketplace_path(&pkg.marketplace);
                let _plugin_entry = manifest.plugins.get(&pkg.name).ok_or_else(|| {
                    Error::PluginNotInManifest(pkg.name.clone())
                })?;

                // Get the source path from the marketplace.json
                let repo = resolver.ensure_marketplace(&pkg.marketplace, &marketplace.url)?;
                resolver.checkout_commit(&repo, &pkg.marketplace, &pkg.marketplace_commit)?;
                let mkt_json = resolver.parse_marketplace_json(&repo, &pkg.marketplace)?;
                let plugin_info = resolver.find_plugin(&mkt_json, &pkg.marketplace, &pkg.name)?;

                let source_path = plugin_info.path.as_ref().ok_or_else(|| {
                    Error::PluginNotFound {
                        plugin: pkg.name.clone(),
                        marketplace: pkg.marketplace.clone(),
                    }
                })?;

                cache.extract_local_plugin(
                    &marketplace_path,
                    source_path,
                    &pkg.marketplace,
                    &pkg.name,
                    &pkg.plugin_commit,
                )?
            }
            SourceType::External => {
                // For external plugins, the repo is already cloned during resolution
                let plugin_repo_path = cache
                    .cache_dir()
                    .join("plugin-repos")
                    .join(&pkg.marketplace)
                    .join(&pkg.name);

                cache.extract_external_plugin(
                    &plugin_repo_path,
                    &pkg.marketplace,
                    &pkg.name,
                    &pkg.plugin_commit,
                )?
            }
        };

        // Register with Claude Code
        claude.add_installed_plugin(
            &pkg.name,
            &pkg.marketplace,
            &install_path,
            &pkg.resolved_version,
            &pkg.plugin_commit,
        )?;

        claude.enable_plugin(&pkg.name, &pkg.marketplace)?;

        installed_count += 1;
    }

    println!("\nInstalled {} plugin(s)", installed_count);
    Ok(())
}

/// Resolve all marketplaces and plugins to create lock file entries.
fn resolve_all(
    manifest: &Manifest,
    resolver: &MarketplaceResolver,
) -> Result<(Vec<LockedMarketplace>, Vec<LockedPackage>)> {
    let mut locked_marketplaces = Vec::new();
    let mut locked_packages = Vec::new();

    // First, resolve all marketplaces
    for (name, entry) in &manifest.marketplaces {
        println!("  Resolving marketplace '{}'...", name);

        let repo = resolver.ensure_marketplace(name, &entry.url)?;

        let commit = if let Some(ref c) = entry.commit {
            c.clone()
        } else if let Some(ref tag) = entry.tag {
            resolver.resolve_tag(&repo, name, tag)?
        } else {
            resolver.resolve_head(&repo)?
        };

        // Checkout the resolved commit
        resolver.checkout_commit(&repo, name, &commit)?;

        locked_marketplaces.push(LockedMarketplace {
            name: name.clone(),
            url: entry.url.clone(),
            commit,
        });
    }

    // Then, resolve all plugins
    for (plugin_name, plugin_entry) in &manifest.plugins {
        println!("  Resolving plugin '{}'...", plugin_name);

        let marketplace = locked_marketplaces
            .iter()
            .find(|m| m.name == plugin_entry.marketplace)
            .ok_or_else(|| Error::UndeclaredMarketplace(plugin_entry.marketplace.clone()))?;

        // Get marketplace info
        let repo = resolver.ensure_marketplace(&marketplace.name, &marketplace.url)?;
        resolver.checkout_commit(&repo, &marketplace.name, &marketplace.commit)?;

        let mkt_json = resolver.parse_marketplace_json(&repo, &marketplace.name)?;
        let plugin_info = resolver.find_plugin(&mkt_json, &marketplace.name, plugin_name)?;

        // Resolve the plugin
        let resolved = resolver.resolve_plugin(
            &marketplace.name,
            &marketplace.commit,
            plugin_name,
            plugin_info,
            plugin_entry.tag.as_deref(),
            plugin_entry.commit.as_deref(),
        )?;

        locked_packages.push(LockedPackage {
            name: resolved.name,
            marketplace: resolved.marketplace,
            source_type: resolved.source_type,
            marketplace_commit: resolved.marketplace_commit,
            plugin_commit: resolved.plugin_commit,
            resolved_version: resolved.resolved_version,
        });
    }

    Ok((locked_marketplaces, locked_packages))
}
