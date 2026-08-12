//! Native Vue prop discovery, type assertions, and authored-range mappings.
//!
//! The element table these checks resolve through — `__VizeNativeElements`,
//! `__VizeNativeElement` and `__VizeNativeElementProp` — is declared once per
//! program in `vue_type_helpers_text!` (`virtual_ts/helpers.rs`), not per file,
//! for two reasons.
//!
//! `import('vue').NativeElements` is named there and nowhere else, behind a
//! `// @ts-ignore`. A Vue whose typings do not export it (Vue 2.7, an older
//! Vue 3 minor, a trimmed or shimmed package, a workspace with no `vue` at all)
//! makes that one alias an error type, which TypeScript propagates as `any`, so
//! every check below it accepts any value and the file degrades to unchecked.
//! Naming the module per file, or per use site, reported `TS2307`/`TS2694` on
//! correct code.
//!
//! The `// @ts-ignore` on the alias is load-bearing, not cosmetic. Without it
//! the alias still degrades to `any`, but it *also* emits a real `TS2694`
//! against the helpers file. The diagnostics path drops that, because the
//! helpers text carries no mapping back to authored source, so the degrade
//! looked complete — but declaration emit (`vize check --declaration`) shells
//! out to Corsa and treats any non-zero exit as fatal, so the same unmapped
//! diagnostic aborted the whole emit with `corsa error (exit code 1)`.
//!
//! The neighbouring `type __Ref<T> = import('vue').Ref<T>;` needs no guard:
//! `Ref` predates every Vue version vize supports.
//!
//! `declare function __vizeNativeElementProp` exists solely to reference the two
//! conditional aliases. They are otherwise named only by the checks below, so a
//! program whose files bind no native prop left them unused and Corsa reported
//! `TS6196` — `'__VizeNativeElement' is declared but never used.` — which reached
//! `check-server` clients as an unmapped hint on a clean SFC. `@ts-ignore` does
//! not help here: it filters errors, not the suggestion channel these arrive on.
//! An ambient `declare function` is exempt from unused reporting itself, which is
//! why `__vForList` can keep `__VForEntry` alive the same way. It is never called.
//!
//! Declaring the conditional types once per program also lets TypeScript reuse
//! its instantiation cache across every generated module instead of
//! re-instantiating `keyof __VizeNativeElements` per file.
//!
//! They are deliberately absent from `DECLARATION_HELPERS_DTS`: emitted
//! declarations never contain a native prop check, so shipping the alias would
//! push vize's Vue-version floor onto every consumer of the emitted `.d.ts`.

use super::super::types::{VizeMapping, VizeSubSpan};
use vize_carton::{CompactString, FxHashMap, String, append, is_native_tag};
use vize_croquis::TemplateExpression;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

pub(crate) type NativePropBindings = FxHashMap<(u32, u32), NativePropBinding>;

pub(crate) struct NativePropBinding {
    tag: CompactString,
    name: CompactString,
    name_start: u32,
    name_end: u32,
}

pub(crate) fn collect_native_prop_bindings(
    root: Option<&RootNode<'_>>,
    enabled: bool,
) -> NativePropBindings {
    let mut bindings = NativePropBindings::default();
    let Some(root) = root.filter(|_| enabled) else {
        return bindings;
    };
    for child in &root.children {
        collect_child_bindings(child, &mut bindings);
    }
    bindings
}

fn collect_child_bindings(child: &TemplateChildNode<'_>, bindings: &mut NativePropBindings) {
    match child {
        TemplateChildNode::Element(element) => collect_element_bindings(element, bindings),
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect_child_bindings(child, bindings);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect_child_bindings(child, bindings);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect_child_bindings(child, bindings);
            }
        }
        _ => {}
    }
}

fn collect_element_bindings(element: &ElementNode<'_>, bindings: &mut NativePropBindings) {
    if is_native_tag(element.tag.as_str()) && !renders_as_fragment(element) {
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            if let Some((range, binding)) = native_prop_binding(element, directive) {
                bindings.insert(range, binding);
            }
        }
    }
    for child in &element.children {
        collect_child_bindings(child, bindings);
    }
}

/// Whether this `<template>` stands for a fragment or a slot body rather than
/// for the DOM element of the same name.
///
/// `template` is an HTML tag, so `<template v-for="x in xs" :key="x.text">`
/// resolved its `key` through the element table, where Vue's own
/// `ReservedProps` declares `key?: PropertyKey | undefined`. A nullable or
/// object key is then `TS2322` — the Vize-only shape #4149 reports from
/// Voicevox's `ToolBar.vue` and Elk's `CommonTabs.vue`.
///
/// The dialect never applies that contract there. `vue-tsc` type-checks a
/// `<template>`'s props only on the path that treats it as an element; a
/// `<template>` carrying `v-for`, `v-if`/`v-else-if`/`v-else` or `v-slot` is
/// compiled as a fragment or a slot body, whose props reach no element type at
/// all — `<template v-for="x in xs" :id="1">` is as silent there as the key is.
/// A bare `<template :id="1">` still renders the element and stays checked, in
/// both tools.
///
/// The key expression itself keeps being emitted, as an unchecked
/// `void (...)` statement, so an invalid member inside it is still `TS2339` at
/// the authored column and the expression stays out of props and event
/// inference exactly as before.
fn renders_as_fragment(element: &ElementNode<'_>) -> bool {
    element.tag == "template"
        && element.props.iter().any(|prop| match prop {
            PropNode::Directive(directive) => matches!(
                directive.name.as_str(),
                "for" | "if" | "else-if" | "else" | "slot"
            ),
            PropNode::Attribute(_) => false,
        })
}

/// Every statically named `v-bind` on a native element is a checkable binding.
///
/// `vue-tsc` checks each one against the element's own prop type — `<a
/// :href="1">` is `TS2322` exactly like `<button :disabled="'yes'">` — so the
/// name is not filtered. An attribute the element type does not declare
/// (`:data-id`, `:aria-label`, a custom attribute) resolves through
/// `__VizeNativeElementProp` to `unknown`, which accepts any value, so the
/// unchecked attributes stay unchecked without a name list to maintain.
///
/// A dynamic name (`:[key]="x"`) has no statically known prop to check against
/// and is skipped, as is `v-bind="object"`, which carries no argument at all.
fn native_prop_binding(
    element: &ElementNode<'_>,
    directive: &DirectiveNode<'_>,
) -> Option<((u32, u32), NativePropBinding)> {
    if directive.name != "bind" {
        return None;
    }
    let ExpressionNode::Simple(argument) = directive.arg.as_ref()? else {
        return None;
    };
    if !argument.is_static {
        return None;
    }
    let expression = directive.exp.as_ref()?;
    let expression_location = expression.loc();
    Some((
        (
            expression_location.start.offset,
            expression_location.end.offset,
        ),
        NativePropBinding {
            tag: element.tag.clone(),
            name: argument.content.clone(),
            name_start: argument.loc.start.offset,
            name_end: argument.loc.end.offset,
        },
    ))
}

pub(super) fn generate_native_prop_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    native_prop: &NativePropBinding,
    generated_expression: &str,
    template_offset: u32,
    indent: &str,
) {
    let name_src_start = (template_offset + native_prop.name_start) as usize;
    let name_src_end = (template_offset + native_prop.name_end) as usize;
    let value_src_start = (template_offset + expr.start) as usize;
    let value_src_end = (template_offset + expr.end) as usize;
    let gen_stmt_start = ts.len();

    append!(*ts, "{indent}const ");
    let check_name_start = ts.len();
    append!(*ts, "__vize_native_prop_check_{}", expr.start);
    let check_name_end = ts.len();
    ts.push_str(": __VizeNativeElementProp<__VizeNativeElement<");
    push_ts_string_literal(ts, native_prop.tag.as_str());
    ts.push_str(">, ");
    push_ts_string_literal(ts, native_prop.name.as_str());
    ts.push_str("> = ");
    let check_anchor_start = ts.len();
    ts.push('(');
    let check_anchor_end = ts.len();
    let value_gen_start = ts.len();
    ts.push_str(generated_expression);
    let value_gen_end = ts.len();
    ts.push_str(");\n");
    let gen_stmt_end = ts.len();
    append!(
        *ts,
        "{indent}void __vize_native_prop_check_{}; // VBind\n",
        expr.start
    );

    mappings.push(VizeMapping {
        gen_range: gen_stmt_start..gen_stmt_end,
        src_range: name_src_start..name_src_end,
        sub_spans: vec![
            VizeSubSpan {
                gen_range: check_name_start..check_name_end,
                src_range: name_src_start..name_src_end,
            },
            VizeSubSpan {
                gen_range: value_gen_start..value_gen_end,
                src_range: value_src_start..value_src_end,
            },
            VizeSubSpan {
                // TypeScript anchors an assignment mismatch on this synthetic
                // wrapper. Keep it after the initializer because batch offset
                // mapping treats a sub-span end as inclusive; the shared
                // boundary must continue to prefer the authored expression.
                gen_range: check_anchor_start..check_anchor_end,
                src_range: name_src_start..name_src_end,
            },
        ],
    });
}

fn push_ts_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}
