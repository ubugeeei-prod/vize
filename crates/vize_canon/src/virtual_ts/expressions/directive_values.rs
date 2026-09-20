//! Resolve custom directive bindings and check their original hook signatures.
//!
//! Calling the hook preserves generic relationships between tuple members and
//! callback parameters. Extracting `Directive<El, Value>` first erases those
//! relationships. Only authored value ranges participate in diagnostics.

use super::super::types::VizeMapping;
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
    let source = (template_offset + expr.start) as usize..(template_offset + expr.end) as usize;
    let name = vize_carton::cstr!("__vize_directive_check_{}", expr.start);
    append!(
        *ts,
        "{indent}const {name} = __vizeDirective({});\n{indent}{name}(null!, {{ ...__vizeDirectiveBindingRest, ",
        binding.variable.as_str()
    );
    let key_start = ts.len();
    ts.push_str("value");
    mappings.push(VizeMapping {
        gen_range: key_start..ts.len(),
        src_range: source.clone(),
        sub_spans: Vec::new(),
    });
    ts.push_str(": (");
    let value_start = ts.len();
    ts.push_str(generated_expression);
    mappings.push(VizeMapping {
        gen_range: value_start..ts.len(),
        src_range: source,
        sub_spans: Vec::new(),
    });
    append!(
        *ts,
        ") }}, ...__vizeDirectiveTail({name})); // CustomDirective\n"
    );
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
