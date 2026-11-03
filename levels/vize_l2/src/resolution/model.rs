//! Model-specific semantic use of the one actual original Identifier.

use super::{BindingLookup, ResolutionError, ResolutionErrorKind, ResolutionTable, sink};
use crate::lang::js::NativeModelInput;
use vize_l0::Span;

pub(crate) fn resolve_model_facts<'a>(
    input: &NativeModelInput<'a>,
    bindings: &impl BindingLookup,
) -> Result<ResolutionTable<'a>, ResolutionError> {
    let expression = input.expression().ok_or(ResolutionError {
        span: Span::new(0, 0),
        kind: ResolutionErrorKind::UnsupportedSyntax,
    })?;
    let occurrences = sink::resolve_model(input, bindings)?;
    Ok(ResolutionTable {
        expression: *expression,
        occurrences,
    })
}
