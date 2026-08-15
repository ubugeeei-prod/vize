//! Custom directive value discovery, type assertions, and authored-range
//! mappings.
//!
//! A custom directive's bound value is checked against the `Value` parameter of
//! the directive's declared `Directive<El, Value>` type, which is what `vue-tsc`
//! does: `const vFocus: Directive<HTMLElement, number>` makes
//! `<div v-focus="'nope'" />` a `TS2322` anchored on the authored value.
//!
//! `__VizeDirectiveValue` carries the same two-part guard as the native element
//! table (see the module docs on [`super::native_props`]), and for the same two
//! reasons:
//!
//! * `// @ts-ignore` on the alias, because it names `import('vue').Directive`.
//!   A `vue` whose typings do not export it — Vue 2.7, a trimmed or shimmed
//!   package, a workspace with no `vue` at all — must degrade every directive
//!   value check to unchecked, never report `TS2694`/`TS2307` on correct code.
//! * an ambient `declare function __vizeDirectiveValue` that references the
//!   alias, because `@ts-ignore` filters errors but not the suggestion channel,
//!   and a program whose files bind no custom directive would otherwise reach
//!   `check-server` clients as an unmapped `TS6196` hint on a clean SFC.
//!
//! ## Why the binding must exist before a check is emitted
//!
//! `v-focus` resolves to the setup binding `vFocus` by Vue's own convention,
//! `'v' + capitalize(camelize(name))`. A directive registered globally — through
//! `app.directive('focus', …)`, a plugin, or an Options API `directives` block —
//! has no such binding, and naming it anyway would turn every globally
//! registered directive in the ecosystem into a `TS2304` false positive. So a
//! binding that is not in `BindingMetadata::bindings` is left unchecked, exactly
//! as it is today.
//!
//! ## Argument and modifiers are out of scope
//!
//! `Directive<El, Value, Modifiers, Arg>` keeps `Value` in the same position, so
//! `v-focus:arg.mod="x"` checks its value identically. Checking `arg` and
//! `mod` against their own parameters is separate, larger work.

use super::super::types::{VizeMapping, VizeSubSpan};
use vize_carton::{CompactString, FxHashMap, String, append, camelize, capitalize};
use vize_croquis::TemplateExpression;
use vize_croquis::croquis::BindingMetadata;
use vize_relief::{DirectiveNode, ElementNode, PropNode, RootNode, TemplateChildNode};

pub(crate) type DirectiveValueBindings = FxHashMap<(u32, u32), DirectiveValueBinding>;

pub(crate) struct DirectiveValueBinding {
    /// The setup binding the directive name resolves to, e.g. `vFocus`.
    variable: CompactString,
}

pub(crate) fn collect_directive_value_bindings(
    root: Option<&RootNode<'_>>,
    bindings: &BindingMetadata,
    enabled: bool,
) -> DirectiveValueBindings {
    let mut collected = DirectiveValueBindings::default();
    let Some(root) = root.filter(|_| enabled) else {
        return collected;
    };
    for child in &root.children {
        collect_child_bindings(child, bindings, &mut collected);
    }
    collected
}

fn collect_child_bindings(
    child: &TemplateChildNode<'_>,
    bindings: &BindingMetadata,
    collected: &mut DirectiveValueBindings,
) {
    match child {
        TemplateChildNode::Element(element) => {
            collect_element_bindings(element, bindings, collected)
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect_child_bindings(child, bindings, collected);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect_child_bindings(child, bindings, collected);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect_child_bindings(child, bindings, collected);
            }
        }
        _ => {}
    }
}

fn collect_element_bindings(
    element: &ElementNode<'_>,
    bindings: &BindingMetadata,
    collected: &mut DirectiveValueBindings,
) {
    for prop in &element.props {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        if let Some((range, binding)) = directive_value_binding(directive, bindings) {
            collected.insert(range, binding);
        }
    }
    for child in &element.children {
        collect_child_bindings(child, bindings, collected);
    }
}

fn directive_value_binding(
    directive: &DirectiveNode<'_>,
    bindings: &BindingMetadata,
) -> Option<((u32, u32), DirectiveValueBinding)> {
    if vize_carton::is_builtin_directive(directive.name) {
        return None;
    }
    let expression = directive.exp.as_ref()?;
    let variable = directive_binding_name(directive.name);
    if !bindings.bindings.contains_key(variable.as_str()) {
        return None;
    }
    let location = expression.loc();
    Some((
        (location.span.start, location.span.end),
        DirectiveValueBinding { variable },
    ))
}

/// Vue's own directive resolution convention: `v-focus` -> `vFocus`,
/// `v-my-directive` -> `vMyDirective`.
fn directive_binding_name(name: &str) -> CompactString {
    let mut resolved = String::with_capacity(name.len() + 1);
    resolved.push('v');
    resolved.push_str(capitalize(camelize(name).as_str()).as_str());
    CompactString::new(resolved.as_str())
}

pub(super) fn generate_directive_value_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    binding: &DirectiveValueBinding,
    generated_expression: &str,
    template_offset: u32,
    indent: &str,
) {
    let value_src_start = (template_offset + expr.start) as usize;
    let value_src_end = (template_offset + expr.end) as usize;
    let gen_stmt_start = ts.len();

    append!(*ts, "{indent}const ");
    let check_name_start = ts.len();
    append!(*ts, "__vize_directive_check_{}", expr.start);
    let check_name_end = ts.len();
    append!(
        *ts,
        ": __VizeDirectiveValue<typeof {}> = (",
        binding.variable.as_str()
    );
    let value_gen_start = ts.len();
    ts.push_str(generated_expression);
    let value_gen_end = ts.len();
    ts.push_str(");\n");
    let gen_stmt_end = ts.len();
    append!(
        *ts,
        "{indent}void __vize_directive_check_{}; // CustomDirective\n",
        expr.start
    );

    // TypeScript anchors `TS2322` on an annotated `const` at the declaration
    // name, so the identifier's sub-span maps to the authored *value*. The
    // oracle in #3445 puts the diagnostic at the start of `'nope'`, not at the
    // directive name — the opposite of the native prop check, where `vue-tsc`
    // anchors on the prop name.
    mappings.push(VizeMapping {
        gen_range: gen_stmt_start..gen_stmt_end,
        src_range: value_src_start..value_src_end,
        sub_spans: vec![
            VizeSubSpan {
                gen_range: check_name_start..check_name_end,
                src_range: value_src_start..value_src_end,
            },
            VizeSubSpan {
                gen_range: value_gen_start..value_gen_end,
                src_range: value_src_start..value_src_end,
            },
        ],
    });
}

#[cfg(test)]
mod tests {
    use super::directive_binding_name;

    #[test]
    fn resolves_vue_directive_naming_convention() {
        assert_eq!(directive_binding_name("focus").as_str(), "vFocus");
        assert_eq!(
            directive_binding_name("my-directive").as_str(),
            "vMyDirective"
        );
        assert_eq!(directive_binding_name("a").as_str(), "vA");
    }
}
