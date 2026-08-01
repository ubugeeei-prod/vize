use oxc_allocator::Allocator;
use vize_carton::String;

use crate::{JsxDiagnostic, JsxLang};

pub(super) fn resolve_vnode_factory<'a>(
    pragma: Option<&'a str>,
    active: bool,
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> Option<&'a str> {
    pragma
        .map(str::trim)
        .filter(|pragma| !pragma.is_empty())
        .filter(|_| active)
        .and_then(|pragma| {
            if valid_pragma_expression(pragma) {
                Some(pragma)
            } else {
                diagnostics.push(JsxDiagnostic::error(
                    "Babel JSX pragma must be a valid JavaScript expression",
                    0,
                    0,
                ));
                None
            }
        })
}

fn valid_pragma_expression(pragma: &str) -> bool {
    let mut probe = String::from("const __vize_pragma = (");
    probe.push_str(pragma);
    probe.push_str(");");
    let allocator = Allocator::default();
    !crate::parse_module(&allocator, probe.as_str(), JsxLang::Jsx).has_errors()
}

/// Babel allocates a fresh helper binding when the source already declares
/// `_transformOn`. A conservative source-text check gives the generated module
/// the same collision safety without coupling this option to a second semantic
/// analysis pass.
pub(super) fn collision_free_transform_on_helper(source: &str) -> String {
    let mut helper = String::from("_transformOn");
    while source.contains(helper.as_str()) {
        helper.push('_');
    }
    helper
}
