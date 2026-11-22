//! Original expression walk with occurrence storage owned by this File's arena.
use super::{
    BindingLookup, Occurrence, ReferenceEvent, ReferenceSink, ResolutionError, ResolutionErrorKind,
    walk,
};
use crate::{expr::JsExpr, resolution::ResolutionTable};
use vize_l0::{Allocator, Vec};

pub(crate) fn resolve<'a>(
    allocator: &'a Allocator,
    expression: &JsExpr<'a>,
    bindings: &impl BindingLookup,
) -> Result<ResolutionTable<'a>, ResolutionError> {
    let mut sink = Arena {
        bindings,
        occurrences: Vec::new_in(&allocator),
    };
    walk::expression(expression, &mut sink)?;
    Ok(ResolutionTable {
        expression: *expression,
        occurrences: crate::resolution::Occurrences::Arena(
            sink.occurrences.into_boxed_slice().into_arena_slice(),
        ),
    })
}

struct Arena<'a, 'b, B> {
    bindings: &'b B,
    occurrences: Vec<'a, Occurrence<'a>>,
}

impl<'a, B: BindingLookup> ReferenceSink<'a> for Arena<'a, '_, B> {
    type Checkpoint = usize;
    fn checkpoint(&self) -> usize {
        self.occurrences.len()
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        include!("immediate.rs")
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.occurrences.truncate(checkpoint);
    }
}
