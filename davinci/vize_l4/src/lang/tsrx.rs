//! The one public compile entry for TSRX modules (#6858).
//!
//! [`compile`] parses the module once (L1), lowers every markup region to
//! canonical L2 ops and emits through the L4 targets. The output is
//! TypeScript: host code is copied byte for byte and each markup region is
//! replaced by its emitted code. A module, construct, target or option that is
//! not supported yet is refused with typed diagnostics at authored byte ranges;
//! there is never partial output.

use alloc::{vec, vec::Vec};
use vize_l0::{Allocator, String, cstr};
use vize_l1::markup::tsrx;
use vize_l2::lang::tsrx::{TsrxIssueKind, lower};

/// The emission target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Virtual-DOM render functions.
    Dom,
    /// Server rendering.
    Ssr,
    /// Vapor.
    Vapor,
}

/// Options for one module.
#[derive(Debug, Clone, Copy)]
pub struct CompileOptions<'a> {
    /// The module's file name, recorded in its source map.
    pub filename: &'a str,
    pub target: Target,
    /// Produce a Source Map v3 in [`Compiled::map`].
    pub source_map: bool,
    /// Erase host TypeScript syntax so the output is JavaScript, over the same
    /// single parse. When `false`, host code is copied verbatim (TypeScript).
    pub erase_types: bool,
}

/// The compiled module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
    /// TypeScript (JavaScript with `erase_types`), without JSX.
    pub code: String,
    /// Source Map v3 JSON, when requested.
    pub map: Option<String>,
}

/// Every diagnostic of a refused module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub diagnostics: Vec<Diagnostic>,
}

/// One diagnostic at UTF-8 byte offsets of the module source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: Code,
    pub message: String,
    pub start: u32,
    pub end: u32,
}

/// What a diagnostic reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Code {
    /// A syntax or template-rule error, with the reference compiler's code
    /// (`TSRX2001`, or a shared `TS1012`).
    Syntax(String),
    /// A construct L2 lowering does not support yet.
    Lowering(TsrxIssueKind),
    /// The target has no TSRX emission yet.
    Target(Target),
    /// A construct or option the emission does not support yet.
    Emit(EmitIssue),
}

/// What the emission does not support yet. New kinds are added as targets
/// grow, and kinds are removed as they are lifted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmitIssue {
    /// [`CompileOptions::source_map`] was requested.
    SourceMap,
    /// [`CompileOptions::erase_types`] was requested.
    TypeErasure,
}

/// Compile one `.tsrx` module.
///
/// # Errors
///
/// Returns every syntax error (the first one ends the parse, as in the
/// reference compiler), every unsupported construct, or every unsupported
/// target and option.
pub fn compile(source: &str, options: &CompileOptions<'_>) -> Result<Compiled, Failure> {
    let allocator = Allocator::new();
    let module = tsrx::parse(&allocator, source);
    if !module.is_clean() {
        return Err(Failure {
            diagnostics: module.diagnostics().iter().map(syntax).collect(),
        });
    }
    if let Err(issues) = lower(&allocator, &module) {
        return Err(Failure {
            diagnostics: issues
                .into_iter()
                .map(|issue| Diagnostic {
                    code: Code::Lowering(issue.kind),
                    message: cstr!("{:?} is not supported in TSRX modules yet", issue.kind),
                    start: issue.span.start,
                    end: issue.span.end,
                })
                .collect(),
        });
    }
    let whole = |code: Code, message: String| Diagnostic {
        code,
        message,
        start: 0,
        end: u32::try_from(source.len()).unwrap_or(u32::MAX),
    };
    let mut diagnostics = vec![whole(
        Code::Target(options.target),
        cstr!(
            "{:?} emission of TSRX modules is not supported yet",
            options.target
        ),
    )];
    if options.source_map {
        diagnostics.push(whole(
            Code::Emit(EmitIssue::SourceMap),
            cstr!("source maps of TSRX modules are not supported yet"),
        ));
    }
    if options.erase_types {
        diagnostics.push(whole(
            Code::Emit(EmitIssue::TypeErasure),
            cstr!("type erasure of TSRX modules is not supported yet"),
        ));
    }
    Err(Failure { diagnostics })
}

fn syntax(diagnostic: &oxc_diagnostics::OxcDiagnostic) -> Diagnostic {
    let code = match (&diagnostic.code.scope, &diagnostic.code.number) {
        (Some(scope), Some(number)) => cstr!("{scope}{number}"),
        _ => String::default(),
    };
    let (start, end) = diagnostic.labels.iter().next().map_or((0, 0), |label| {
        let start = label.offset();
        let len = label.len();
        (start, start.saturating_add(len))
    });
    Diagnostic {
        code: Code::Syntax(code),
        message: String::from(diagnostic.message.as_ref()),
        start,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::{Code, CompileOptions, Diagnostic, EmitIssue, Target, compile};
    use vize_l2::lang::tsrx::TsrxIssueKind;

    const OPTIONS: CompileOptions<'static> = CompileOptions {
        filename: "App.tsrx",
        target: Target::Dom,
        source_map: false,
        erase_types: false,
    };

    fn diagnostics(source: &str, options: &CompileOptions<'_>) -> Vec<Diagnostic> {
        compile(source, options).unwrap_err().diagnostics
    }

    #[test]
    fn syntax_errors_carry_the_reference_code_and_offset() {
        // tests/_fixtures/tsrx/errors/code-block-return.tsrx: TSRX2001 at 62..74.
        let source = include_str!("../../../../tests/_fixtures/tsrx/errors/code-block-return.tsrx");
        let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Syntax("TSRX2001".into()));
        assert_eq!((diagnostic.start, diagnostic.end), (62, 74));
    }

    #[test]
    fn unsupported_constructs_are_refused_at_their_span() {
        let source = "export function List({ items }: { items: string[] }) @{\n\t@for (const item of items) {\n\t\t<li>{item}</li>\n\t}\n}\n";
        let [diagnostic] = diagnostics(source, &OPTIONS).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Lowering(TsrxIssueKind::For));
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some("@for (const item of items) {\n\t\t<li>{item}</li>\n\t}")
        );
    }

    #[test]
    fn unsupported_targets_and_options_are_each_refused_over_the_module() {
        let source = "export const App = () => <p class=\"x\">hi</p>;\n";
        let whole = (0, source.len() as u32);
        for target in [Target::Dom, Target::Ssr, Target::Vapor] {
            let options = CompileOptions { target, ..OPTIONS };
            let [diagnostic] = diagnostics(source, &options).try_into().unwrap();
            assert_eq!(diagnostic.code, Code::Target(target));
            assert_eq!((diagnostic.start, diagnostic.end), whole);
        }

        let options = CompileOptions {
            source_map: true,
            erase_types: true,
            ..OPTIONS
        };
        let refused: Vec<_> = diagnostics(source, &options)
            .into_iter()
            .map(|diagnostic| (diagnostic.code, (diagnostic.start, diagnostic.end)))
            .collect();
        assert_eq!(
            refused,
            [
                (Code::Target(Target::Dom), whole),
                (Code::Emit(EmitIssue::SourceMap), whole),
                (Code::Emit(EmitIssue::TypeErasure), whole),
            ]
        );
    }
}
