//! Check the observable inputs of the CSS emitter, class lookup and lexical
//! definition queries in two complete original styles. Spans may move.
use super::{ScssProducts, ScssRefusal};
use vize_l0::{SourceBlock, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScssMismatch {
    Original(ScssRefusal),
    Replacement(ScssRefusal),
    Rules,
    Classes,
    Definitions,
    Uses,
}

/// Both inputs are bound to their exact immutable source blocks. Only the
/// checker can construct this value.
#[derive(Debug)]
pub struct ScssEquivalence<'a, 'b> {
    original: SourceBlock<'a>,
    replacement: SourceBlock<'b>,
}

impl<'a, 'b> ScssEquivalence<'a, 'b> {
    pub fn original(&self) -> SourceBlock<'a> {
        self.original
    }
    pub fn replacement(&self) -> SourceBlock<'b> {
        self.replacement
    }
}

fn spelling(block: SourceBlock<'_>, span: Span) -> Option<&str> {
    if span.start < block.start() || span.end > block.end() {
        return None;
    }
    block
        .root_source()
        .get(span.start as usize..span.end as usize)
}

impl<'a> ScssProducts<'a> {
    /// The L4 emitter consumes each run's group boundary, selector and ordered
    /// property names/values. Compare those exact fields before granting an
    /// edit. A same-CSS edit may still change a shadowed definition or use.
    pub fn check_equivalent<'b>(
        &self,
        next: &ScssProducts<'b>,
    ) -> Result<ScssEquivalence<'a, 'b>, ScssMismatch> {
        let a = self.program().map_err(ScssMismatch::Original)?;
        let b = next.program().map_err(ScssMismatch::Replacement)?;
        if a.runs().len() != b.runs().len() {
            return Err(ScssMismatch::Rules);
        }
        let mut previous_groups = None;
        for (a, b) in a.runs().iter().zip(b.runs()) {
            let boundary_changed = previous_groups
                .is_some_and(|(left, right)| (left != a.group) != (right != b.group));
            previous_groups = Some((a.group, b.group));
            if boundary_changed
                || a.selector != b.selector
                || !a
                    .declarations
                    .iter()
                    .map(|p| (p.name, p.value.as_str()))
                    .eq(b.declarations.iter().map(|p| (p.name, p.value.as_str())))
            {
                return Err(ScssMismatch::Rules);
            }
        }
        let Some(classes) = self.classes().filter(|classes| classes.is_complete()) else {
            return Err(ScssMismatch::Classes);
        };
        if Some(classes) != next.classes() {
            return Err(ScssMismatch::Classes);
        }
        if a.variables().len() != b.variables().len() {
            return Err(ScssMismatch::Definitions);
        }
        for (left, right) in a.variables().iter().zip(b.variables()) {
            if left.name != right.name
                || left.value != right.value
                || spelling(self.block(), left.span) != Some(left.name)
                || spelling(next.block(), right.span) != Some(right.name)
            {
                return Err(ScssMismatch::Definitions);
            }
        }
        if a.variable_uses().len() != b.variable_uses().len() {
            return Err(ScssMismatch::Uses);
        }
        for (left, right) in a.variable_uses().iter().zip(b.variable_uses()) {
            let Some(original) = spelling(self.block(), left.span) else {
                return Err(ScssMismatch::Uses);
            };
            if left.declaration != right.declaration
                || Some(original) != spelling(next.block(), right.span)
            {
                return Err(ScssMismatch::Uses);
            }
        }
        Ok(ScssEquivalence {
            original: self.block(),
            replacement: next.block(),
        })
    }
}
