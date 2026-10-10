//! Direct, original-source slot parameter grammar. No wrapper or retry.

use std::any::Any;

use oxc_ast::ast::FormalParameters;
use oxc_diagnostics::Diagnostics;

use crate::{
    Parser, ParserConfig, ParserDiagnostic, ParserImpl, UniquePromise,
    config::{NoTokensParserConfig, RuntimeParserConfig, TokensParserConfig},
    diagnostics,
    lexer::Kind,
};

impl<'a, C: ParserConfig> Parser<'a, C> {
    /// Parse the entire original source as an arrow-function parameter list,
    /// retaining original spans, binding defaults, rest and TS annotations.
    /// This goal has an EOF delimiter; it adds no synthetic source or AST.
    pub fn parse_slot_parameters(self) -> Result<FormalParameters<'a>, Diagnostics> {
        let config: &dyn Any = &self.config;
        if config.is::<NoTokensParserConfig>() {
            parse_no_tokens(self.allocator, self.source_text, self.source_type, self.options)
        } else if config.is::<TokensParserConfig>() {
            parse_tokens(self.allocator, self.source_text, self.source_type, self.options)
        } else if let Some(&config) = config.downcast_ref::<RuntimeParserConfig>() {
            parse_runtime(self.allocator, self.source_text, self.source_type, self.options, config)
        } else {
            ParserImpl::<C>::new(
                self.allocator,
                self.source_text,
                self.source_type,
                self.options,
                self.config,
                UniquePromise::new(),
            )
            .parse_slot_parameters()
        }
    }
}

// Stock goals remain provider-instantiated, as the ordinary expression entry.
macro_rules! stock_goal {
    ($name:ident, $config:ident) => {
        #[inline(never)]
        fn $name<'a>(
            allocator: &'a oxc_allocator::Allocator,
            source: &'a str,
            source_type: oxc_span::SourceType,
            options: crate::ParseOptions,
        ) -> Result<FormalParameters<'a>, Diagnostics> {
            ParserImpl::<$config>::new(
                allocator,
                source,
                source_type,
                options,
                $config,
                UniquePromise::new(),
            )
            .parse_slot_parameters()
        }
    };
}
stock_goal!(parse_no_tokens, NoTokensParserConfig);
stock_goal!(parse_tokens, TokensParserConfig);

#[inline(never)]
fn parse_runtime<'a>(
    allocator: &'a oxc_allocator::Allocator,
    source: &'a str,
    source_type: oxc_span::SourceType,
    options: crate::ParseOptions,
    config: RuntimeParserConfig,
) -> Result<FormalParameters<'a>, Diagnostics> {
    ParserImpl::<RuntimeParserConfig>::new(
        allocator,
        source,
        source_type,
        options,
        config,
        UniquePromise::new(),
    )
    .parse_slot_parameters()
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    fn parse_slot_parameters(mut self) -> Result<FormalParameters<'a>, Diagnostics> {
        self.bump_any();
        let parameters = self.parse_slot_parameter_items();
        if self.fatal_error.is_none() && !self.at(Kind::Eof) {
            self.error(diagnostics::unexpected_token(self.cur_token().span()));
        }
        if let Some(fatal) = self.fatal_error.take() {
            return Err(fatal.error.into_diagnostic().into());
        }
        self.check_unfinished_errors();
        let errors = self
            .lexer
            .errors
            .into_iter()
            .chain(self.errors)
            .map(ParserDiagnostic::into_diagnostic)
            .collect::<Diagnostics>();
        if errors.is_empty() { Ok(parameters) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests;
