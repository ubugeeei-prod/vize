//! Vue expression globals, below authored template and script bindings.
#![expect(clippy::disallowed_macros, reason = "LSP fields take std `String`")]

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use super::bindings::analyzed_template_binding_completions;
use crate::ide::IdeContext;

// Vue 3.5's public instance properties. `$event` remains scope-local: it is
// supplied by the existing event-handler analysis, never by this table.
const VUE_PROPERTIES: &[(&str, &str)] = &[
    ("$attrs", "Fallthrough attributes"),
    ("$slots", "Slots from parent"),
    ("$refs", "Template refs"),
    ("$el", "Root element"),
    ("$props", "Props object"),
    ("$data", "Component data"),
    ("$options", "Component options"),
    ("$parent", "Parent instance"),
    ("$root", "Root instance"),
    ("$host", "Custom element host"),
];
const VUE_METHODS: &[(&str, &str)] = &[
    ("$emit", "Emit event"),
    ("$forceUpdate", "Schedule a component update"),
    ("$nextTick", "Wait for the next DOM update"),
    ("$watch", "Watch a reactive source"),
];

// Match Vue's globalsAllowList.ts rather than the broader browser/Node scope
// or Croquis's compiler helper list. In particular window/document/require
// and generated render names are not template globals.
const JS_CONSTANTS: &[&str] = &["Infinity", "undefined", "NaN"];
const JS_FUNCTIONS: &[&str] = &[
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
];
const JS_CONSTRUCTORS: &[&str] = &[
    "Number", "Date", "Array", "Object", "Boolean", "String", "RegExp", "Map", "Set", "BigInt",
    "Error", "Symbol",
];
const JS_OBJECTS: &[&str] = &["Math", "JSON", "Intl", "console"];

pub(super) fn template_expression_completions(ctx: &IdeContext<'_>) -> Vec<CompletionItem> {
    let mut items = analyzed_template_binding_completions(ctx, true);
    // Petite-vue has a different instance surface. Member access is already
    // answered by the checker; identifier globals cannot supply its members.
    if ctx.dialect().is_petite_vue()
        || crate::ide::template_expression::is_at_member_access_position(&ctx.content, ctx.offset)
    {
        return items;
    }

    for (name, description) in VUE_PROPERTIES {
        append_global(&mut items, name, CompletionItemKind::PROPERTY, description);
    }
    for (name, description) in VUE_METHODS {
        append_global(&mut items, name, CompletionItemKind::METHOD, description);
    }
    for (names, kind) in [
        (JS_CONSTANTS, CompletionItemKind::CONSTANT),
        (JS_FUNCTIONS, CompletionItemKind::FUNCTION),
        (JS_CONSTRUCTORS, CompletionItemKind::CLASS),
        (JS_OBJECTS, CompletionItemKind::MODULE),
    ] {
        for name in names {
            append_global(
                &mut items,
                name,
                kind,
                "JavaScript global allowed in Vue templates",
            );
        }
    }
    items
}

fn append_global(
    items: &mut Vec<CompletionItem>,
    name: &str,
    kind: CompletionItemKind,
    detail: &str,
) {
    // Keep the whole authored candidate, including its type, documentation,
    // insertion and resolution data, when a local binding shadows a global.
    if items.iter().any(|item| item.label == name) {
        return;
    }
    items.push(CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        sort_text: Some(format!("1{name}")),
        ..Default::default()
    });
}
