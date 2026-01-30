mod cache;
mod claude;

pub use cache::CacheManager;
pub use claude::{ClaudeCodeIntegration, InstalledPluginEntry, InstalledPluginsFile, PluginScope};
