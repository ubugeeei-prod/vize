//! Setup binding anchor emission for template-used names.

use super::imports::collect_setup_binding_anchor_names;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::Croquis;

pub(super) fn emit_setup_binding_anchors(
    ts: &mut String,
    summary: &Croquis,
    script_content: Option<&str>,
    template_referenced_names: Option<&FxHashSet<String>>,
    comment: &str,
) {
    if summary.bindings.bindings.is_empty() {
        return;
    }

    let binding_names =
        collect_setup_binding_anchor_names(summary, script_content, template_referenced_names);
    let mut first = true;
    for name in binding_names {
        if is_reserved_anchor_name(name) {
            continue;
        }
        if first {
            append!(*ts, "\n  // {comment}\n  ");
        } else {
            ts.push(' ');
        }
        append!(*ts, "void {name};");
        first = false;
    }
    if !first {
        ts.push('\n');
    }
}

fn is_reserved_anchor_name(name: &str) -> bool {
    matches!(
        name,
        "default"
            | "class"
            | "new"
            | "delete"
            | "void"
            | "typeof"
            | "in"
            | "instanceof"
            | "return"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "with"
            | "var"
            | "let"
            | "const"
            | "function"
            | "this"
            | "super"
            | "import"
            | "export"
            | "yield"
            | "await"
            | "async"
            | "static"
            | "enum"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}
