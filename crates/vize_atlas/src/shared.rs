//! Shared ownership used by the heterogeneous memoization boundary.

/// Reference-counted value shared by the cache and typed query outcomes.
///
/// Atlas cannot use a scoped borrow here: product values outlive individual
/// provider calls and one memoized dependency may feed multiple root outcomes.
#[allow(clippy::disallowed_types)]
pub type Shared<T> = std::sync::Arc<T>;
