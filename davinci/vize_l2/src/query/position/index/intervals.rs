//! A balanced interval tree in one sorted buffer. The maximum end in each
//! subtree lets a point query skip even long runs under an enclosing scope.

use super::PositionQueryError;
use alloc::vec::Vec;
use vize_l0::Span;

struct Entry<T> {
    span: Span,
    max_end: u32,
    value: T,
}

pub(in crate::query) struct Intervals<T>(Vec<Entry<T>>);

impl<T> Default for Intervals<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl Intervals<usize> {
    pub(super) fn from_rows<R>(rows: &[R], span: impl Fn(&R) -> Span) -> Self {
        let mut entries = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let at = span(row);
            if !at.is_empty() {
                entries.push(Entry::new(at, index));
            }
        }
        Self::sorted(entries)
    }
}

impl<T> Entry<T> {
    fn new(span: Span, value: T) -> Self {
        Self {
            span,
            max_end: span.end,
            value,
        }
    }
}

impl<T> Intervals<T> {
    pub(in crate::query) fn try_from_sites<E>(
        collect: impl FnOnce(&mut dyn FnMut(Span, T)) -> Result<(), E>,
    ) -> Result<Self, E> {
        Self::try_from_sized_sites(0, collect)
    }

    pub(in crate::query) fn try_from_sized_sites<E>(
        capacity: usize,
        collect: impl FnOnce(&mut dyn FnMut(Span, T)) -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut entries = Vec::with_capacity(capacity);
        collect(&mut |span, value| {
            if !span.is_empty() {
                entries.push(Entry::new(span, value));
            }
        })?;
        Ok(Self::sorted(entries))
    }

    fn sorted(mut entries: Vec<Entry<T>>) -> Self {
        entries.sort_unstable_by_key(|entry| entry.span.start);
        mark_subtrees(&mut entries);
        Self(entries)
    }

    pub(in crate::query) fn visit<E>(
        &self,
        offset: u32,
        visit: &mut impl FnMut(Span, &T) -> Result<(), E>,
    ) -> Result<(), E> {
        point(&self.0, offset, visit)
    }

    pub(in crate::query) fn unique<'s, R>(
        &'s self,
        offset: u32,
        mut select: impl FnMut(Span, &'s T) -> Option<R>,
    ) -> Result<Option<R>, PositionQueryError> {
        let mut result = None;
        point(&self.0, offset, &mut |span, value| {
            if let Some(row) = select(span, value) {
                if result.is_some() {
                    return Err(PositionQueryError::AmbiguousSite);
                }
                result = Some(row);
            }
            Ok(())
        })?;
        Ok(result)
    }
}

fn mark_subtrees<T>(rows: &mut [Entry<T>]) -> u32 {
    let (left, tail) = rows.split_at_mut(rows.len() / 2);
    let Some((middle, right)) = tail.split_first_mut() else {
        return 0;
    };
    middle.max_end = middle
        .span
        .end
        .max(mark_subtrees(left))
        .max(mark_subtrees(right));
    middle.max_end
}

fn point<'a, T, E>(
    rows: &'a [Entry<T>],
    offset: u32,
    visit: &mut impl FnMut(Span, &'a T) -> Result<(), E>,
) -> Result<(), E> {
    let (left, tail) = rows.split_at(rows.len() / 2);
    let Some((middle, right)) = tail.split_first() else {
        return Ok(());
    };
    if middle.max_end <= offset {
        return Ok(());
    }
    point(left, offset, visit)?;
    if middle.span.start <= offset {
        if offset < middle.span.end {
            visit(middle.span, &middle.value)?;
        }
        point(right, offset, visit)?;
    }
    Ok(())
}
