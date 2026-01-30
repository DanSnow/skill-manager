mod add;
mod init;
mod install;
mod list;
mod remove;

use clap::{Parser, Subcommand};

use crate::Result;

#[derive(Parser)]
#[command(name = "skill-manager")]
#[command(about = "Reproducible plugin management for Claude Code")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new plugins.toml manifest
    Init {
        /// Create global manifest (~/.config/skill-manager/plugins.toml)
        #[arg(long)]
        global: bool,
    },

    /// Add a plugin to the manifest
    Add {
        /// Plugin name
        name: String,

        /// Marketplace to use
        #[arg(long)]
        marketplace: Option<String>,

        /// Pin to a specific tag
        #[arg(long)]
        tag: Option<String>,

        /// Pin to a specific commit
        #[arg(long)]
        commit: Option<String>,
    },

    /// Install plugins from the manifest
    Install {
        /// Re-resolve all versions and update the lock file
        #[arg(long)]
        update: bool,

        /// Prefer global versions when conflicts occur
        #[arg(long, conflicts_with = "prefer_project")]
        prefer_global: bool,

        /// Prefer project versions when conflicts occur
        #[arg(long, conflicts_with = "prefer_global")]
        prefer_project: bool,
    },

    /// Remove a plugin from the manifest
    Remove {
        /// Plugin name to remove
        name: String,
    },

    /// List installed plugins
    List,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Init { global } => init::run(global),
            Commands::Add {
                name,
                marketplace,
                tag,
                commit,
            } => add::run(name, marketplace, tag, commit),
            Commands::Install {
                update,
                prefer_global,
                prefer_project,
            } => install::run(update, prefer_global, prefer_project),
            Commands::Remove { name } => remove::run(name),
            Commands::List => list::run(),
        }
    }
}
