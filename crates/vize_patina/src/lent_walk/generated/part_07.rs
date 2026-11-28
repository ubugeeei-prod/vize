// Generated from OXC's pinned immutable walk; see mod.rs for provenance.
use super::super::LentNodes;
use super::{walk_span, walk_ts_index_signature_name};
use oxc_allocator::ArenaVec;
use oxc_ast::ast::{Span, TSIndexSignatureName};

#[inline]
pub(crate) fn walk_ts_index_signature_names<'a, V: LentNodes<'a>>(
    visitor: &mut V,
    it: &'a ArenaVec<'a, TSIndexSignatureName<'a>>,
) {
    for el in it {
        walk_ts_index_signature_name(visitor, el);
    }
}

#[inline]
pub(crate) fn walk_spans<'a, V: LentNodes<'a>>(visitor: &mut V, it: &'a ArenaVec<'a, Span>) {
    for el in it {
        walk_span(visitor, el);
    }
}
