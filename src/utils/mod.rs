//! Utility modules.

mod icons;
mod process_info;

pub use icons::{create_icon_cache, CachedIcon, IconCache, IconCacheStats, SharedIconCache};
pub use process_info::ProcessInfoProvider;

#[cfg(test)]
#[path = "icons_test.rs"]
mod icons_test;

#[cfg(test)]
#[path = "process_info_test.rs"]
mod process_info_test;
