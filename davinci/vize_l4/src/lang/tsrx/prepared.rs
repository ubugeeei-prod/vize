//! Consume canonical TSRX regions for a typed emission request.
use super::{Code, CompileOptions, Compiled, Diagnostic, EmitIssue, Failure, Target, dom, syntax};
use alloc::vec::Vec;
use vize_l0::String;
use vize_l0::cstr;
use vize_l2::lang::tsrx::prepared::{PreparedTsrx, RegionLowering};

/// Emit from the same source and sealed regions that analysis tools have read.
/// Syntax and region lowering take precedence over target and capability refusals.
pub fn compile_prepared(
    prepared: PreparedTsrx<'_>,
    options: &CompileOptions<'_>,
) -> Result<Compiled, Failure> {
    let module = prepared.module();
    let arena = prepared.arena();
    let artifacts = match prepared.into_regions() {
        RegionLowering::Syntax => {
            return Err(Failure {
                diagnostics: module.diagnostics().iter().map(syntax).collect(),
            });
        }
        RegionLowering::Refused(issues) => {
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
        RegionLowering::Ready(artifacts) => artifacts,
    };
    let whole = |code: Code, message: String| Diagnostic {
        code,
        message,
        start: 0,
        end: u32::try_from(module.program().source_text.len()).unwrap_or(u32::MAX),
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
    let code = dom::module(arena, module, artifacts)?;
    #[cfg(feature = "typescript")]
    let code = if options.erase_types {
        super::erase::erase(&code).map_err(|message| Failure {
            diagnostics: alloc::vec![whole(Code::Emit(EmitIssue::TypeErasure), message)],
        })?
    } else {
        code
    };
    Ok(Compiled { code, map: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::tsrx::compile;
    use vize_l2::lang::tsrx::prepared::with_prepared;

    #[test]
    fn target_capabilities_and_stage_refusal_precedence_match_the_compiler() {
        for source in [
            "export function Card<T>({x}: {x: T}) @{ <b>{x}</b> }",
            "export const bad = <a></b>;",
            "export function Refused() @{ <p>one &amp; two</p> }",
            "export const Outside = <i />;",
        ] {
            for target in [Target::Dom, Target::Ssr, Target::Vapor] {
                for source_map in [false, true] {
                    for erase_types in [false, true] {
                        let options = CompileOptions {
                            filename: "Original.tsrx",
                            target,
                            source_map,
                            erase_types,
                        };
                        let observed =
                            with_prepared(source, |stages| compile_prepared(stages, &options));
                        assert_eq!(observed, compile(source, &options), "{options:?}");
                    }
                }
            }
        }
    }
}
