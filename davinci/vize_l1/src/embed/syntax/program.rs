//! Ordinary whole-Program syntax with an explicit compiler source profile.

use oxc_diagnostics::Diagnostics;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_l0::Allocator;

use super::{EmbedHole, EmbedSource, Grammar, Lang, NativeSyntax, Shape, coordinates::Coordinates};

/// An authored program's goal, selected before parsing. No Unambiguous retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramGoal {
    Module,
    Script,
}

/// Explicit JS/TS and JSX/TSX compiler-profile identity.
///
/// File/container language resolution is a separate caller responsibility.
/// Formatter options are not admitted by this entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramOptions {
    pub lang: Lang,
    pub jsx: bool,
    pub goal: ProgramGoal,
}

impl ProgramOptions {
    #[must_use]
    pub const fn module(lang: Lang) -> Self {
        Self {
            lang,
            jsx: false,
            goal: ProgramGoal::Module,
        }
    }

    #[must_use]
    pub const fn source_type(self) -> SourceType {
        let source_type = match self.lang {
            Lang::Js => SourceType::mjs(),
            Lang::Ts => SourceType::ts(),
        }
        .with_jsx(self.jsx);
        match self.goal {
            ProgramGoal::Module => source_type.with_module(true),
            ProgramGoal::Script => source_type.with_script(true),
        }
    }
}

/// Parse the original Program once through the pinned production OXC library.
///
/// Source, actual comments and full normally owned diagnostics survive local
/// holes. The complete Program has no synthetic wrapper or unit-count cap.
/// Syntax success does not claim semantic validation or hostile-input quotas.
pub fn parse_program_once<'a>(
    allocator: &'a Allocator,
    source: EmbedSource<'a>,
    options: ProgramOptions,
) -> NativeSyntax<'a> {
    let source_type = options.source_type();
    let mut syntax = NativeSyntax {
        grammar: Grammar {
            shape: Shape::Program,
            lang: options.lang,
        },
        source_type,
        coordinates: Coordinates { source, prefix: 0 },
        program: None,
        diagnostics: Diagnostics::default(),
        hole: None,
    };
    if super::parser_length(source.text().len(), 0).is_none() {
        syntax.hole = Some(EmbedHole::SourceTooLarge);
        return syntax;
    }
    let parsed = parse_input(allocator, source.text(), source_type);
    syntax.hole = if parsed.is_flow_language {
        Some(EmbedHole::UnsupportedFlow)
    } else if parsed.panicked || parsed.diagnostics.has_errors() {
        Some(EmbedHole::Syntax)
    } else {
        None
    };
    syntax.diagnostics = parsed.diagnostics;
    syntax.program = Some(parsed.program);
    syntax
}

/// All native syntax paths use the same single parser invocation.
pub(super) fn parse_input<'a>(
    allocator: &'a Allocator,
    input: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    Parser::new(allocator.as_oxc(), input, source_type).parse()
}

#[cfg(test)]
mod tests;
