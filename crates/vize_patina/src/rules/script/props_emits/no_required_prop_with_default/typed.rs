//! Typed macro defaults, including types declared in a sibling script block.

use oxc_ast::ast::Program;

use super::{LintDiagnostic, META, ScriptLintResult};
use crate::rules::script::props_emits::typed_prop_defaults::collect_setup_props_with_types;

pub(super) fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    sibling: Option<(&'a Program<'a>, usize)>,
    result: &mut ScriptLintResult,
) {
    for declaration in collect_setup_props_with_types(program, offset, sibling) {
        for prop in declaration.typed.iter().filter(|prop| prop.required) {
            for default in declaration.default_spans(&prop.name) {
                let base = prop.offset as u32;
                let default_base = offset as u32;
                result.add_diagnostic(
                    LintDiagnostic::error(
                        META.name,
                        vize_l0::cstr!(
                            "Prop '{}' is required but also declares a default",
                            prop.name
                        ),
                        base + prop.span.start,
                        base + prop.span.end,
                    )
                    .with_label(
                        "marked required here",
                        base + prop.key_span.start,
                        base + prop.key_span.end,
                    )
                    .with_label(
                        "default declared here",
                        default_base + default.start,
                        default_base + default.end,
                    )
                    .with_help("Make the typed prop optional with `?` or remove the default."),
                );
            }
        }
    }
}
