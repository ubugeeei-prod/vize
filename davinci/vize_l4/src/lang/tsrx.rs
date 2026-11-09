//! The one public compile entry for TSRX modules (#6858).
//!
//! [`compile`] parses the module once (L1), lowers every markup region to
//! canonical L2 ops and emits through the L4 targets. A module, construct or
//! target that is not supported yet is refused with typed diagnostics at
//! authored byte ranges; there is never partial output.

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
    /// The module's file name, for diagnostics and source maps.
    pub filename: &'a str,
    pub target: Target,
    /// Whether to produce a Source Map v3.
    pub source_map: bool,
    /// Erase host TypeScript syntax so the output is JavaScript, over the same
    /// single parse. When `false`, host code is copied verbatim (TypeScript).
    pub erase_types: bool,
}

/// The compiled module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
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
    /// Host type erasure is not supported yet.
    TypeErasure,
}

/// Compile one `.tsrx` module.
///
/// # Errors
///
/// Returns every syntax error (the first one ends the parse, as in the
/// reference compiler), every unsupported construct, or the unsupported target.
pub fn compile(source: &str, options: &CompileOptions<'_>) -> Result<Compiled, Failure> {
    let allocator = Allocator::new();
    let module = tsrx::parse(&allocator, source);
    if !module.is_clean() {
        return Err(Failure {
            diagnostics: module.diagnostics().iter().map(syntax).collect(),
        });
    }
    let whole = u32::try_from(source.len()).unwrap_or(u32::MAX);
    match lower(&allocator, &module) {
        Err(issues) => Err(Failure {
            diagnostics: issues
                .into_iter()
                .map(|issue| Diagnostic {
                    code: Code::Lowering(issue.kind),
                    message: cstr!("{:?} is not supported in TSRX modules yet", issue.kind),
                    start: issue.span.start,
                    end: issue.span.end,
                })
                .collect(),
        }),
        Ok(_) => {
            let mut diagnostics = vec![Diagnostic {
                code: Code::Target(options.target),
                message: cstr!(
                    "{:?} emission of TSRX modules is not supported yet",
                    options.target
                ),
                start: 0,
                end: whole,
            }];
            if options.erase_types {
                diagnostics.push(Diagnostic {
                    code: Code::TypeErasure,
                    message: cstr!("type erasure of TSRX modules is not supported yet"),
                    start: 0,
                    end: whole,
                });
            }
            Err(Failure { diagnostics })
        }
    }
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
    use super::{Code, CompileOptions, Diagnostic, Target, compile};
    use vize_l2::lang::tsrx::TsrxIssueKind;

    const OPTIONS: CompileOptions<'static> = CompileOptions {
        filename: "App.tsrx",
        target: Target::Dom,
        source_map: false,
        erase_types: false,
    };

    fn diagnostics(source: &str) -> Vec<Diagnostic> {
        compile(source, &OPTIONS).unwrap_err().diagnostics
    }

    #[test]
    fn syntax_errors_carry_the_reference_code_and_offset() {
        // tests/_fixtures/tsrx/errors/code-block-return.tsrx: TSRX2001 at 62..74.
        let source = include_str!("../../../../tests/_fixtures/tsrx/errors/code-block-return.tsrx");
        let [diagnostic] = diagnostics(source).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Syntax("TSRX2001".into()));
        assert_eq!((diagnostic.start, diagnostic.end), (62, 74));
    }

    #[test]
    fn unsupported_constructs_and_targets_are_refused_without_code() {
        let source = "export function List({ items }: { items: string[] }) @{\n\t@for (const item of items) {\n\t\t<li>{item}</li>\n\t}\n}\n";
        let [diagnostic] = diagnostics(source).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Lowering(TsrxIssueKind::For));
        assert!(
            source
                .get(diagnostic.start as usize..)
                .is_some_and(|rest| rest.starts_with("@for"))
        );

        let source = "export const App = () => <p class=\"x\">hi</p>;\n";
        let [diagnostic] = diagnostics(source).try_into().unwrap();
        assert_eq!(diagnostic.code, Code::Target(Target::Dom));
        assert_eq!((diagnostic.start, diagnostic.end), (0, source.len() as u32));

        let options = CompileOptions {
            erase_types: true,
            ..OPTIONS
        };
        let codes: Vec<_> = compile(source, &options)
            .unwrap_err()
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert_eq!(codes, [Code::Target(Target::Dom), Code::TypeErasure]);
    }
}
