//! Analysis results stored beside a tree instead of on fat nodes.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use core::marker::PhantomData;

use crate::id::NodeId;

/// A [`NodeId`]-keyed table. Dense vs hashed storage is decided by #6869.
#[derive(Debug, Clone)]
pub struct SideTable<T> {
    _values: PhantomData<T>,
}

impl<T> SideTable<T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        todo!()
    }

    /// Store `value` for `id`, returning the previous value.
    pub fn insert(&mut self, id: NodeId, value: T) -> Option<T> {
        let _ = (id, value);
        todo!()
    }

    /// The value stored for `id`.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        let _ = id;
        todo!()
    }

    /// Mutable access to the value stored for `id`.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let _ = id;
        todo!()
    }

    /// Remove and return the value stored for `id`.
    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let _ = id;
        todo!()
    }

    /// Number of stored values.
    #[must_use]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True when nothing is stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

impl<T> Default for SideTable<T> {
    fn default() -> Self {
        Self::new()
    }
}
