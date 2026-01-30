## 1. Project Setup

- [x] 1.1 Initialize Cargo project with `cargo init`
- [x] 1.2 Add dependencies: clap (derive), toml, toml_edit, serde (derive), serde_json, git2, xdg, thiserror, rootcause
- [x] 1.3 Create module structure: config/, resolver/, installer/, cli/
- [x] 1.4 Define Error enum in lib.rs with thiserror

## 2. Config Module - Manifest Parsing

- [x] 2.1 Define Manifest struct with marketplaces and plugins fields
- [x] 2.2 Implement marketplace URL parsing (GitHub shorthand, SSH, HTTPS)
- [x] 2.3 Implement marketplace inline table parsing (url, tag, commit)
- [x] 2.4 Implement plugin entry parsing (marketplace, tag, commit)
- [x] 2.5 Add manifest file location resolution (global vs project)
- [x] 2.6 Add tests for manifest parsing

## 3. Config Module - Lock File

- [x] 3.1 Define LockFile struct with marketplace and package arrays
- [x] 3.2 Implement lock file serialization with header comment
- [x] 3.3 Implement lock file deserialization
- [x] 3.4 Add tests for lock file round-trip

## 4. Resolver Module - Marketplace Operations

- [x] 4.1 Implement marketplace clone to cache directory
- [x] 4.2 Implement marketplace fetch for existing clones
- [x] 4.3 Implement tag resolution to commit hash
- [x] 4.4 Implement checkout for specific commit
- [x] 4.5 Parse marketplace.json to discover plugins
- [x] 4.6 Add tests for marketplace resolution

## 5. Resolver Module - Plugin Resolution

- [x] 5.1 Implement local plugin resolution (path in marketplace)
- [x] 5.2 Implement external plugin resolution (separate git URL)
- [x] 5.3 Implement plugin version pinning (tag/commit)
- [x] 5.4 Read plugin.json for resolved_version
- [x] 5.5 Add tests for plugin resolution

## 6. Installer Module - Cache Management

- [x] 6.1 Implement XDG directory resolution for config and cache
- [x] 6.2 Create CACHEDIR.TAG in cache directory
- [x] 6.3 Implement plugin extraction to cache with commit-based paths
- [x] 6.4 Implement skip logic for already-extracted plugins
- [x] 6.5 Add tests for cache operations

## 7. Installer Module - Claude Code Integration

- [x] 7.1 Implement reading existing installed_plugins.json
- [x] 7.2 Implement writing plugin entries to installed_plugins.json
- [x] 7.3 Implement reading existing settings.json
- [x] 7.4 Implement enabling plugins in settings.json
- [x] 7.5 Add tests for Claude Code file operations

## 8. CLI - Init Command

- [x] 8.1 Implement `init` subcommand with clap
- [x] 8.2 Add `--global` flag for global manifest
- [x] 8.3 Create empty plugins.toml at appropriate location
- [x] 8.4 Handle existing manifest error case

## 9. CLI - Add Command

- [x] 9.1 Implement `add` subcommand with plugin name argument
- [x] 9.2 Add `--marketplace`, `--tag`, `--commit` options
- [x] 9.3 Use toml_edit to preserve manifest formatting
- [x] 9.4 Implement marketplace search when not specified
- [x] 9.5 Prompt to add unknown marketplace

## 10. CLI - Install Command

- [x] 10.1 Implement `install` subcommand
- [x] 10.2 Add `--update` flag to re-resolve versions
- [x] 10.3 Implement lock file-based installation (reproducible)
- [x] 10.4 Implement fresh resolution when no lock file
- [x] 10.5 Display installation progress
- [x] 10.6 Handle global+project manifest combination

## 11. CLI - Remove Command

- [x] 11.1 Implement `remove` subcommand with plugin name argument
- [x] 11.2 Use toml_edit to remove plugin from manifest
- [x] 11.3 Handle non-existent plugin error

## 12. CLI - List Command

- [x] 12.1 Implement `list` subcommand
- [x] 12.2 Display plugin name, marketplace, version, lock status
- [x] 12.3 Indicate version drift from lock file
- [x] 12.4 Handle empty manifest case

## 13. Conflict Handling

- [x] 13.1 Implement conflict detection between global and project
- [x] 13.2 Display conflict information with Claude Code behavior note
- [x] 13.3 Implement interactive resolution prompt (update/skip/abort)
- [x] 13.4 Add `--prefer-global` and `--prefer-project` flags
- [x] 13.5 Add tests for conflict scenarios

## 14. Error Handling & Polish

- [x] 14.1 Ensure all error types have clear messages
- [x] 14.2 Add `--help` documentation for all commands
- [x] 14.3 Verify error chain display with rootcause
- [x] 14.4 Add integration tests for full install workflow
