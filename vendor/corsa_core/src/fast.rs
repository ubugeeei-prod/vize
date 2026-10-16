//! Fast-path collection and string aliases shared across the workspace.
//!
//! The bindings lean on these re-exports in hot paths so higher-level crates can
//! opt into compact strings, inline buffers, and fast hash maps without having to
//! repeat dependency choices.

use std::fmt::{self, Write as _};

pub use compact_str::{CompactString, ToCompactString};
pub use memchr::{memchr, memmem};
pub use rustc_hash::{FxHashMap, FxHashSet};
pub use smallvec::SmallVec;

/// Hash map alias tuned for internal lookups.
///
/// # Examples
///
/// ```no_run
/// use corsa_core::fast::FastMap;
///
/// let mut map = FastMap::default();
/// map.insert("answer", 42);
/// assert_eq!(map.get("answer"), Some(&42));
/// ```
pub type FastMap<K, V> = FxHashMap<K, V>;

/// Hash set alias tuned for internal membership checks.
///
/// # Examples
///
/// ```no_run
/// use corsa_core::fast::FastSet;
///
/// let mut set = FastSet::default();
/// set.insert("corsa");
/// assert!(set.contains("corsa"));
/// ```
pub type FastSet<T> = FxHashSet<T>;

/// Formats into a [`CompactString`] without going through [`String`].
///
/// # Examples
///
/// ```no_run
/// use corsa_core::fast::compact_format;
///
/// let value = compact_format(format_args!("node-{}/{}", 1, "leader"));
/// assert_eq!(value.as_str(), "node-1/leader");
/// ```
pub fn compact_format(args: fmt::Arguments<'_>) -> CompactString {
    let mut value = CompactString::default();
    let _ = value.write_fmt(args);
    value
}
