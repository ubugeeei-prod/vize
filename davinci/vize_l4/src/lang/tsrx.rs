//! The one public compile entry for TSRX modules (#6858).
//!
//! [`compile`] parses the module once (L1), lowers every markup region to
//! canonical L2 ops and emits through the L4 targets. The output is
//! TypeScript: host code is copied byte for byte and each markup region is
//! replaced by its emitted code. A module, construct, target or option that is
//! not supported yet is refused with typed diagnostics at authored byte ranges;
//! there is never partial output.
//!
//! The DOM target emits Vue components ([`dom`]). SSR and Vapor are refused.
//! With the `typescript` feature, [`CompileOptions::erase_types`] erases the
//! compiled module's TypeScript; without it, the option is refused.

use alloc::{vec, vec::Vec};
use vize_l0::{Allocator, String, cstr};
use vize_l1::markup::tsrx;
use vize_l2::lang::tsrx::{TsrxIssueKind, lower};

mod component;
mod dom;
#[cfg(feature = "typescript")]
mod erase;
mod names;
mod props;
mod scopes;

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
    /// Erase TypeScript syntax so the output is JavaScript, as the repository's
    /// `.vue` script erasure prints it. When `false`, host code is copied
    /// verbatim (TypeScript).
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
    /// [`CompileOptions::source_map`] was requested. The shared DOM emitter
    /// records no span links, so render code would stay unmapped.
    SourceMap,
    /// [`CompileOptions::erase_types`] was requested without the
    /// `typescript` feature, or the compiled module could not be erased.
    TypeErasure,
    /// DOM: markup outside the body or the returned value of a top-level
    /// component, a function whose name starts with an uppercase letter, or
    /// held by a value other than a conditional's arm, the right side of `&&`
    /// or a `.map()` callback's result (`{render(<b />)}`).
    OutsideComponent,
    /// DOM: parameters other than none, one identifier, or one object pattern
    /// of static keys, each bound to a name with an optional default the
    /// reference types, and an optional rest name (see `props.rs`).
    Parameters,
    /// DOM: the props identifier read as a value in an expression-bodied
    /// arrow, or a destructured prop used as a component tag (#13280).
    PropsValue,
    /// DOM: a destructured prop's local declared again in the component's own
    /// scope (its setup outside nested functions and blocks). Nested scopes
    /// shadow it as in JavaScript.
    ShadowedProp,
    /// DOM: an `async` or generator component.
    AsyncComponent,
    /// DOM: a component body without a rendered node.
    NoRender,
    /// DOM: a `return` in a component's setup before its render, which would
    /// pick the output once, at setup.
    EarlyReturn,
    /// DOM: a host name equal to one the target writes (`__props`, `_ctx`, a
    /// helper alias).
    GeneratedName,
    /// DOM: the shared DOM emitter refused the region.
    Dom,
}

impl Failure {
    fn emit(issue: EmitIssue, span: vize_l0::Span) -> Self {
        Self {
            diagnostics: vec![Diagnostic {
                code: Code::Emit(issue),
                message: cstr!("{issue:?} is not emitted for the DOM target yet"),
                start: span.start,
                end: span.end,
            }],
        }
    }
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
    let artifacts = match lower(&allocator, &module) {
        Ok(artifacts) => artifacts,
        Err(issues) => {
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
    };
    let whole = |code: Code, message: String| Diagnostic {
        code,
        message,
        start: 0,
        end: u32::try_from(source.len()).unwrap_or(u32::MAX),
    };
    let mut diagnostics = Vec::new();
    if options.target != Target::Dom {
        diagnostics.push(whole(
            Code::Target(options.target),
            cstr!(
                "{:?} emission of TSRX modules is not supported yet",
                options.target
            ),
        ));
    }
    if options.source_map {
        diagnostics.push(whole(
            Code::Emit(EmitIssue::SourceMap),
            cstr!("source maps of TSRX modules are not supported yet"),
        ));
    }
    if options.erase_types && !cfg!(feature = "typescript") {
        diagnostics.push(whole(
            Code::Emit(EmitIssue::TypeErasure),
            cstr!("type erasure of TSRX modules needs the `typescript` feature"),
        ));
    }
    if !diagnostics.is_empty() {
        return Err(Failure { diagnostics });
    }
    let code = dom::module(&allocator, &module, artifacts)?;
    #[cfg(feature = "typescript")]
    let code = if options.erase_types {
        erase::erase(&code).map_err(|message| Failure {
            diagnostics: vec![whole(Code::Emit(EmitIssue::TypeErasure), message)],
        })?
    } else {
        code
    };
    Ok(Compiled { code, map: None })
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
mod tests;
