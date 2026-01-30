# skill-manager

Reproducible plugin management for Claude Code.

## Overview

skill-manager brings Cargo-like dependency management to Claude Code plugins. It uses a human-editable `plugins.toml` manifest and an auto-generated `plugins.lock` file to ensure reproducible plugin installations across machines and team members.

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize a project manifest
skill-manager init

# Add a plugin
skill-manager add superpowers --marketplace official

# Install plugins
skill-manager install
```

## Commands

### `init`

Create a new `plugins.toml` manifest.

```bash
# Project manifest (.claude/plugins.toml)
skill-manager init

# Global manifest (~/.config/skill-manager/plugins.toml)
skill-manager init --global
```

### `add`

Add a plugin to the manifest.

```bash
# Basic usage
skill-manager add <plugin-name> --marketplace <marketplace>

# Pin to a specific tag
skill-manager add superpowers --marketplace official --tag v4.1.1

# Pin to a specific commit
skill-manager add sourceatlas --marketplace official --commit abc123
```

### `install`

Install plugins from the manifest.

```bash
# Install using lock file (reproducible)
skill-manager install

# Re-resolve versions and update lock file
skill-manager install --update

# Conflict resolution flags
skill-manager install --prefer-global
skill-manager install --prefer-project
```

### `remove`

Remove a plugin from the manifest.

```bash
skill-manager remove <plugin-name>
```

Note: This only removes the plugin from the manifest. Run `skill-manager install` to sync.

### `list`

Show installed plugins with their status.

```bash
skill-manager list
```

## Configuration

### Manifest (`plugins.toml`)

```toml
[marketplaces]
# GitHub shorthand
official = "anthropics/claude-plugins-official"

# SSH URL (for private repos)
private = "git@github.com:mycompany/plugins.git"

# HTTPS URL
custom = "https://git.example.com/plugins.git"

# Pin marketplace to a tag
pinned = { url = "owner/repo", tag = "v1.0" }

# Pin marketplace to a commit
exact = { url = "owner/repo", commit = "abc123def456" }

[plugins]
# Basic plugin
typescript-lsp = { marketplace = "official" }

# Pin plugin to a tag
superpowers = { marketplace = "official", tag = "v4.1.1" }

# Pin plugin to a commit
sourceatlas = { marketplace = "official", commit = "def456" }
```

### Lock File

The `plugins.lock` file is auto-generated and pins exact versions. Commit it to version control for reproducible installations. Do not edit manually.

## File Locations

| File | Location |
|------|----------|
| Global manifest | `~/.config/skill-manager/plugins.toml` |
| Global lock file | `~/.config/skill-manager/plugins.lock` |
| Project manifest | `.claude/plugins.toml` |
| Project lock file | `.claude/plugins.lock` |
| Cache | `~/.cache/skill-manager/` |

## Global vs Project

- **Global plugins** (`~/.config/skill-manager/`): Available in all projects
- **Project plugins** (`.claude/`): Project-specific, can override global

When both exist, skill-manager processes both manifests. Conflicts (same plugin, different versions) can be resolved with `--prefer-global` or `--prefer-project` flags.

## License

MIT
