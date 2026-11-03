//! Caller-provided message lookup, independent of locale and catalog storage.

use alloc::borrow::Cow;

/// Supplies message text for a stable diagnostic key.
///
/// A provider may borrow embedded text or return owned text. The result must
/// outlive the key and provider, so diagnostics cannot retain a temporary
/// catalog or caller buffer. Locale selection and fallback belong to the
/// provider; consumers do not consult a global catalog.
pub trait MessageLookup {
    /// Look up `key` using this provider's locale and fallback policy.
    fn lookup(&self, key: &str) -> Cow<'static, str>;
}
