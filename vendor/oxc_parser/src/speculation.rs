//! Share failed parser subproblems across overlapping type-argument probes.
//!
//! A failure is reusable only inside the reversible expression type-argument
//! transaction. It never substitutes a result in the committed parse. Keys
//! retain the complete token (including lexical mode/relex flags), previous
//! token end, grammar parameters and nontransactional state. Successful parses
//! are never memoized: their AST and side effects must be produced normally.

use oxc_allocator::{Dummy, GetAllocator};

use crate::{
    Context, ParserConfig, ParserImpl,
    diagnostics::ParserDiagnostic,
    lexer::{Token, trivia_builder::SpeculativeTrivia},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SpeculativeProduction {
    Type,
    Assignment { allow_return_type_in_arrow_function: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Effects {
    grammar: u64,
    module_record: u64,
    trivia: u64,
    script_errors: usize,
    module_errors: usize,
    encountered_await_identifier: bool,
    potential_await_reparse: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FailedSubproblemKey {
    production: SpeculativeProduction,
    token: Token,
    lexer_position: (u32, Token),
    previous_token_end: u32,
    context: Context,
    effects: Effects,
    trivia: SpeculativeTrivia,
}

pub(crate) struct FailedSubproblem<'a> {
    error: ParserDiagnostic<'a>,
    token: Token,
    previous_token_end: u32,
    trivia: SpeculativeTrivia,
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    #[inline]
    fn speculative_effects(&self) -> Effects {
        Effects {
            grammar: self.state.speculation_effects_revision,
            module_record: self.module_record_builder.speculation_effects_revision,
            trivia: self.lexer.trivia_builder.speculation_effects_revision,
            script_errors: self.deferred_script_errors.len(),
            module_errors: self.lexer.deferred_module_errors.len(),
            encountered_await_identifier: self.state.encountered_await_identifier,
            potential_await_reparse: self.state.potential_await_reparse.len(),
        }
    }

    #[inline]
    pub(crate) fn memoize_failed_speculative_subproblem<T: Dummy<'a>>(
        &mut self,
        production: SpeculativeProduction,
        parse: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if self.state.type_argument_speculation_depth == 0 || self.fatal_error.is_some() {
            return parse(self);
        }
        #[cfg(test)]
        {
            self.state.subproblem_visits += 1;
            if self.state.disable_failed_subproblems {
                return parse(self);
            }
        }
        if !self.state.reuse_failed_type_argument_subproblems {
            return parse(self);
        }
        let key = FailedSubproblemKey {
            production,
            token: self.token,
            lexer_position: self.lexer.speculative_position(),
            previous_token_end: self.prev_token_end,
            context: self.ctx,
            effects: self.speculative_effects(),
            trivia: self.lexer.trivia_builder.speculative_state(),
        };
        if let Some(failed) = self.state.failed_speculative_subproblems.get(&key) {
            let error = failed.error.clone();
            self.token = failed.token;
            self.prev_token_end = failed.previous_token_end;
            self.lexer.trivia_builder.restore_speculative_state(failed.trivia);
            self.set_fatal_error(error);
            #[cfg(test)]
            {
                self.state.subproblem_reuses += 1;
            }
            return Dummy::dummy(self.allocator());
        }
        let value = parse(self);
        // Parser checkpoints deliberately retain some state (comments, module
        // records, cover grammar tables and deferred diagnostics). Never skip
        // a subproblem which wrote any of it. Monotonic revisions also detect
        // insert/remove pairs whose final collection lengths happen to match.
        if key.effects == self.speculative_effects()
            && let Some(fatal) = &self.fatal_error
        {
            self.state.failed_speculative_subproblems.insert(
                key,
                FailedSubproblem {
                    error: fatal.error.clone(),
                    token: self.token,
                    previous_token_end: self.prev_token_end,
                    trivia: self.lexer.trivia_builder.speculative_state(),
                },
            );
        }
        value
    }
}

#[cfg(test)]
mod tests;
