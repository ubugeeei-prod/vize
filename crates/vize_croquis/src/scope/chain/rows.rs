//! The scope table: its rows and the row the chain is currently in.
//!
//! [`super::ScopeChain`] reads the current row on every identifier lookup, so
//! that read skips the bounds check. It may skip it because the table keeps
//! one law: the current row exists. The fields are private to this module,
//! every constructor starts with a root row selected, rows are only appended,
//! and a new selection is checked against the table.
//!
//! The law is the type invariant `current < rows.len()`. Creusot proves that
//! every method below keeps it and that both unchecked reads meet
//! `get_unchecked`'s precondition (see `creusot.rs` beside this file). No
//! method has a precondition, so the proof holds whatever the rest of the
//! crate passes in.

#[cfg(creusot)]
use creusot_std::prelude::*;

/// Scope rows plus the index of the current row.
pub(crate) struct Rows<T> {
    rows: Vec<T>,
    current: usize,
}

#[cfg(creusot)]
impl<T> Invariant for Rows<T> {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.current@ < self.rows@.len() }
    }
}

impl<T> Rows<T> {
    /// The rows of this table, in order.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn rows(self) -> Seq<T> {
        pearlite! { self.rows@ }
    }

    /// The index of the current row.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn selected(self) -> Int {
        pearlite! { self.current@ }
    }

    /// The table of `rows` with the first row current, or `rows` back when
    /// there is none. The caller builds the rows in place (`vec![root]`), so
    /// the root is written straight into its allocation.
    #[cfg_attr(creusot, ensures(rows@.len() == 0 ==> result == Err(rows)))]
    #[cfg_attr(creusot, ensures(rows@.len() > 0 ==> exists<table: Self> result == Ok(table) && table.rows() == rows@ && table.selected() == 0))]
    #[inline]
    pub(crate) fn from_rows(rows: Vec<T>) -> Result<Self, Vec<T>> {
        // The slice's `is_empty`: creusot-std specifies it, not `Vec`'s.
        if <[T]>::is_empty(&rows) {
            Err(rows)
        } else {
            Ok(Self { rows, current: 0 })
        }
    }

    /// A table holding only `root`, which is current, with room for
    /// `capacity` rows.
    #[cfg_attr(creusot, ensures(result.rows() == Seq::singleton(root)))]
    #[cfg_attr(creusot, ensures(result.selected() == 0))]
    #[inline]
    pub(crate) fn with_capacity(capacity: usize, root: T) -> Self {
        let mut rows = Vec::with_capacity(capacity);
        rows.push(root);
        Self { rows, current: 0 }
    }

    /// The number of rows.
    #[cfg_attr(creusot, ensures(result@ == self.rows().len()))]
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    /// The index of the current row. The chain mints no id past `u32::MAX`,
    /// so the narrowing keeps every index it can select.
    #[cfg_attr(creusot, ensures(self.selected() <= u32::MAX@ ==> result@ == self.selected()))]
    #[inline]
    pub(crate) const fn current_index(&self) -> u32 {
        self.current as u32
    }

    /// Every row, in order.
    #[cfg_attr(creusot, ensures(result@ == self.rows()))]
    #[inline]
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.rows
    }

    /// Every row, in order, for in-place edits. The table keeps its length
    /// and its current row.
    #[cfg_attr(creusot, ensures(result@ == self.rows()))]
    #[cfg_attr(creusot, ensures((^result)@ == (^self).rows()))]
    #[cfg_attr(creusot, ensures((^self).selected() == self.selected()))]
    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.rows
    }

    /// The row at `index`, if the table has one there.
    #[cfg_attr(creusot, ensures(index@ < self.rows().len() ==> exists<row: &T> result == Some(row) && *row == self.rows()[index@]))]
    #[cfg_attr(creusot, ensures(self.rows().len() <= index@ ==> result == None))]
    #[inline]
    pub(crate) fn get(&self, index: u32) -> Option<&T> {
        self.rows.get(index as usize)
    }

    /// The row at `index` for an in-place edit, if the table has one there.
    #[cfg_attr(creusot, ensures(self.rows().len() <= index@ ==> result == None))]
    #[cfg_attr(creusot, ensures((^self).rows().len() == self.rows().len()))]
    #[cfg_attr(creusot, ensures((^self).selected() == self.selected()))]
    #[inline]
    pub(crate) fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.rows.get_mut(index as usize)
    }

    /// Make room for `additional` more rows without changing any.
    #[cfg_attr(creusot, ensures((^self).rows() == self.rows()))]
    #[cfg_attr(creusot, ensures((^self).selected() == self.selected()))]
    #[inline]
    pub(crate) fn reserve_exact(&mut self, additional: usize) {
        self.rows.reserve_exact(additional);
    }

    /// Append `row` and make it current.
    #[cfg_attr(creusot, ensures((^self).rows() == self.rows().push_back(row)))]
    #[cfg_attr(creusot, ensures((^self).selected() == self.rows().len()))]
    #[inline]
    pub(crate) fn push_selected(&mut self, row: T) {
        let selected = self.rows.len();
        self.rows.push(row);
        self.current = selected;
    }

    /// Make the row at `index` current if the table has one there, and say
    /// whether it did. A missing row leaves the current row unchanged.
    #[cfg_attr(creusot, ensures(result == (index@ < self.rows().len())))]
    #[cfg_attr(creusot, ensures((^self).rows() == self.rows()))]
    #[cfg_attr(creusot, ensures(result ==> (^self).selected() == index@))]
    #[cfg_attr(creusot, ensures(!result ==> (^self).selected() == self.selected()))]
    #[inline]
    pub(crate) fn select(&mut self, index: u32) -> bool {
        let index = index as usize;
        let exists = index < self.rows.len();
        if exists {
            self.current = index;
        }
        exists
    }

    /// The current row.
    #[cfg_attr(creusot, ensures(*result == self.rows()[self.selected()]))]
    #[inline]
    pub(crate) fn current(&self) -> &T {
        // SAFETY: the type invariant `current < rows.len()` holds for every
        // value of this type (Creusot proof, see the module comment), so the
        // index addresses an initialized row.
        unsafe { self.rows.get_unchecked(self.current) }
    }

    /// The current row for an in-place edit. The table keeps its length and
    /// its current row, and `&mut self` excludes any other borrow of the rows.
    #[cfg_attr(creusot, ensures(*result == self.rows()[self.selected()]))]
    #[cfg_attr(creusot, ensures((^self).rows() == self.rows().set(self.selected(), ^result)))]
    #[cfg_attr(creusot, ensures((^self).selected() == self.selected()))]
    #[inline]
    pub(crate) fn current_mut(&mut self) -> &mut T {
        // SAFETY: the type invariant `current < rows.len()`, as in `current`.
        unsafe { self.rows.get_unchecked_mut(self.current) }
    }
}

#[cfg(test)]
mod tests {
    use super::Rows;

    #[test]
    fn a_failed_push_keeps_the_current_row_in_bounds() {
        // A zero-sized vector reaches the capacity-overflow panic without
        // allocating. Unwinding must leave the table's previous row selected.
        let mut table = Rows::from_rows(vec![(); usize::MAX]).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            table.push_selected(());
        }));
        assert!(result.is_err());
        assert_eq!(table.current, 0);
        assert_eq!(table.rows.len(), usize::MAX);
    }
}
