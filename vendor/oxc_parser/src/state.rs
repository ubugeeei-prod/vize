use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;

use crate::{Context, cursor::ParserCheckpoint};

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    /// Failed expression type-argument probes survive rewinds so nested malformed
    /// generics do not repeatedly allocate the same speculative AST. Grammar
    /// context is part of the key because parameter initializers can change it.
    pub not_type_arguments_in_expression: FxHashSet<(u32, Context)>,

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
            not_parenthesized_arrow: FxHashSet::default(),
            not_type_arguments_in_expression: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
        }
    }
}
