use crate::config::Manifest;
use crate::{Error, Result};

/// Create a new plugins.toml manifest.
pub fn run(global: bool) -> Result<()> {
    let path = if global {
        Manifest::global_path().ok_or_else(|| {
            Error::CacheCreate(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine global config directory",
            ))
        })?
    } else {
        Manifest::project_path()
    };

    if path.exists() {
        return Err(Error::ManifestExists(path));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::FileWrite {
            path: path.clone(),
            source: e,
        })?;
    }

    // Create empty manifest
    let content = r#"# skill-manager plugins manifest
# See https://github.com/example/skill-manager for documentation

[marketplaces]
# Add marketplace sources here
# official = "anthropics/claude-plugins-official"

[plugins]
# Add plugins here
# superpowers = { marketplace = "official" }
"#;

    std::fs::write(&path, content).map_err(|e| Error::FileWrite {
        path: path.clone(),
        source: e,
    })?;

    println!("Created {}", path.display());
    Ok(())
}
