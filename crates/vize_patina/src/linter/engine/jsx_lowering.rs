//! Select the original source factory only when an active rule needs it.

use crate::markup::L2Markup;
use std::ops::Deref;
use vize_atelier_jsx::{JsxLang, LowerOutput, LoweredRoot, SyntaxLowerOutput, SyntaxRoot};
use vize_croquis::Croquis;
use vize_l0::Allocator;

pub(super) enum Lowered<'a> {
    Plain(LowerOutput<'a>),
    Syntax(SyntaxLowerOutput<'a>),
}

pub(super) fn lower<'a>(
    arena: &'a Allocator,
    parser: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
    retain_syntax: bool,
) -> Lowered<'a> {
    if retain_syntax {
        Lowered::Syntax(vize_atelier_jsx::lower_source_with_lint_syntax(
            arena, parser, source, lang,
        ))
    } else {
        Lowered::Plain(vize_atelier_jsx::lower_source(arena, parser, source, lang))
    }
}

pub(super) struct Root<'o, 'a> {
    lowered: &'o LoweredRoot<'a>,
    syntax: Option<SyntaxRoot<'o, 'a>>,
}

impl<'a> Deref for Root<'_, 'a> {
    type Target = LoweredRoot<'a>;

    fn deref(&self) -> &Self::Target {
        self.lowered
    }
}

impl<'o, 'a: 'o> Root<'o, 'a> {
    pub(super) fn markup(&self) -> Option<L2Markup<'o>> {
        match self.syntax {
            Some(syntax) => L2Markup::from_syntax_root(syntax),
            None => self
                .lowered
                .l2
                .as_ref()
                .ok()
                .map(L2Markup::from_projected_root),
        }
    }
}

impl<'a> Lowered<'a> {
    pub(super) fn analysis(&self) -> &Croquis {
        match self {
            Self::Plain(output) => &output.analysis,
            Self::Syntax(output) => output.analysis(),
        }
    }

    pub(super) fn diagnostics(&self) -> &[vize_atelier_jsx::diagnostics::JsxDiagnostic] {
        match self {
            Self::Plain(output) => &output.diagnostics,
            Self::Syntax(output) => output.diagnostics(),
        }
    }

    pub(super) fn roots(&self) -> impl Iterator<Item = Root<'_, 'a>> {
        let plain = match self {
            Self::Plain(output) => Some(output.roots.iter()),
            _ => None,
        };
        let syntax = match self {
            Self::Syntax(output) => Some(output.roots()),
            _ => None,
        };
        plain
            .into_iter()
            .flatten()
            .map(|lowered| Root {
                lowered,
                syntax: None,
            })
            .chain(syntax.into_iter().flatten().map(|syntax| Root {
                lowered: syntax.lowered(),
                syntax: Some(syntax),
            }))
    }
}
