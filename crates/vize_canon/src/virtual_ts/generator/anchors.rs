//! Setup binding anchor emission for template-used names.

use super::imports::collect_setup_binding_anchor_names;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{BindingType, Croquis};

/// Header comment for the anchor list; the wording differs when unused-local
/// diagnostics are preserved because the list is then narrowed to
/// template-referenced names.
pub(super) fn setup_binding_anchor_comment(preserve_unused_diagnostics: bool) -> &'static str {
    if preserve_unused_diagnostics {
        "Reference setup bindings used by template generation"
    } else {
        "Reference setup bindings (used in template/CSS v-bind)"
    }
}

/// Anchor a setup binding named `props` at setup scope, before the template
/// closure opens.
///
/// The closure's synthetic `const props` shadows a setup binding of the same
/// name, so the in-closure anchor list can no longer mark it read — a
/// `const props = defineProps(...)` used only through the template
/// (`v-bind="props"`) reported a false TS6133.
///
/// Only a template that spells `props` itself counts as a read, so a component
/// whose template reads prop names directly (`{{ label }}`) keeps its genuine
/// unused-`props` report. Unlike the in-closure anchor list this narrowing is
/// unconditional: the LSP generates with unused-local diagnostics unpreserved
/// yet still surfaces TS6133 as a hint, so anchoring on binding kind alone
/// would suppress true positives there.
pub(super) fn emit_props_shadow_anchor(
    ts: &mut String,
    summary: &Croquis,
    template_referenced_names: &FxHashSet<String>,
) {
    if summary
        .bindings
        .bindings
        .get("props")
        .is_some_and(is_setup_variable)
        && template_referenced_names
            .iter()
            .any(|name| name.as_str() == "props")
    {
        ts.push_str("  // Anchor before the template scope shadows `props`\n");
        ts.push_str("  void props;\n");
    }
}

/// True for bindings declared as variables in `<script setup>` scope
/// (let/const/ref/reactive), as opposed to props, Options API members,
/// globals, or imports.
fn is_setup_variable(kind: &BindingType) -> bool {
    matches!(
        kind,
        BindingType::SetupLet
            | BindingType::SetupMaybeRef
            | BindingType::SetupRef
            | BindingType::SetupReactiveConst
            | BindingType::SetupConst
    )
}

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
