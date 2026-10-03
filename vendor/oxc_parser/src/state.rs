use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;

use crate::{Context, cursor::ParserCheckpoint};

pub struct ParserState<'a> {
    /// Failed probes survive rewinds. Share the existing arrow-probe table so
    /// ordinary parses incur no additional table initialization or drop cost.
    pub failed_speculations: FxHashSet<FailedSpeculation>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression` and `ObjectExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression` / `ObjectExpression`.
    /// Valued by position of the trailing_comma.
    pub trailing_commas: FxHashMap<u32, Span>,

    /// Statements that may need reparsing when `sourceType` is `unambiguous`.
    ///
    /// In unambiguous mode, we initially parse top-level `await ...` as
    /// `await(...)` (identifier/function call). But if ESM syntax is detected
    /// later, we need to reparse these as await expressions.
    ///
    /// Each entry contains: (statement_index, checkpoint_before_statement)
    pub potential_await_reparse: Vec<(usize, ParserCheckpoint<'a>)>,

    /// Flag to track if an `await` identifier was encountered during statement parsing.
    /// Used to determine if a statement needs to be stored for potential reparsing
    /// in unambiguous mode.
    pub encountered_await_identifier: bool,
}

impl ParserState<'_> {
    pub fn new() -> Self {
        Self {
            failed_speculations: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
        }
    }
}

/// One-word keys keep arrow probes cheap while isolating type-argument probes
/// and their grammar contexts. Source offsets use only the lower 32 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FailedSpeculation(u64);

impl FailedSpeculation {
    #[inline]
    pub fn parenthesized_arrow(position: u32) -> Self {
        Self(u64::from(position))
    }

    #[inline]
    pub fn type_arguments(position: u32, context: Context) -> Self {
        Self((1_u64 << 63) | (u64::from(context.bits()) << 32) | u64::from(position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_probe_kinds_and_grammar_contexts_are_independent() {
        let mut state = ParserState::new();
        let key = FailedSpeculation::type_arguments(u32::MAX, Context::In);
        state.failed_speculations.insert(key);
        assert!(state.failed_speculations.contains(&key));
        assert!(
            !state
                .failed_speculations
                .contains(&FailedSpeculation::parenthesized_arrow(u32::MAX))
        );
        assert!(
            !state
                .failed_speculations
                .contains(&FailedSpeculation::type_arguments(
                    u32::MAX,
                    Context::In | Context::Yield,
                ))
        );
    }
}
