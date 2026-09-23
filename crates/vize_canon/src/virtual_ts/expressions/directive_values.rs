//! Resolve custom directive bindings and check their original hook signatures.
//!
//! Calling the hook preserves generic relationships between tuple members and
//! callback parameters. Extracting `Directive<El, Value>` first erases those
//! relationships. Only authored value ranges participate in diagnostics.

use super::super::types::VizeMapping;
use crate::virtual_ts::script_facts::ScriptBindings;
use vize_carton::{CompactString, FxHashMap, String, append, camelize, capitalize};
use vize_croquis::TemplateExpression;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

pub(crate) type DirectiveValueBindings = FxHashMap<(u32, u32), DirectiveValueBinding>;

pub(crate) struct DirectiveValueBinding {
    /// Vue's resolution name of the directive, e.g. `vFocus`.
    pub(super) variable: CompactString,
    /// Whether `variable` is a setup binding. Any other directive is one the
    /// component registers (`directives`) or the application does
    /// (`GlobalDirectives`).
    pub(super) in_setup: bool,
    /// The static argument and its template range: `v-dir:arg`.
    pub(super) arg: Option<(CompactString, (u32, u32))>,
    /// Each modifier and its template range: `v-dir.a.b`.
    pub(super) modifiers: Vec<(CompactString, (u32, u32))>,
    /// Whether the classic `<script>` default export is in scope as
    /// `__default__`, the home of a component's own `directives` option.
    pub(super) has_default_alias: bool,
    /// A Vapor component calls the directive itself, with a value getter,
    /// instead of a hook that receives a binding object.
    vapor: bool,
}

pub(crate) fn collect_directive_value_bindings(
    root: Option<&RootNode<'_>>,
    bindings: ScriptBindings<'_>,
    (has_default_alias, vapor): (bool, bool),
    enabled: bool,
) -> DirectiveValueBindings {
    let mut collected = DirectiveValueBindings::default();
    let Some(root) = root.filter(|_| enabled) else {
        return collected;
    };
    for child in &root.children {
        collect_child_bindings(child, bindings, &mut collected);
    }
    for binding in collected.values_mut() {
        binding.has_default_alias = has_default_alias;
        binding.vapor = vapor;
    }
    collected
}

fn collect_child_bindings(
    child: &TemplateChildNode<'_>,
    bindings: ScriptBindings<'_>,
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
    bindings: ScriptBindings<'_>,
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
    bindings: ScriptBindings<'_>,
) -> Option<((u32, u32), DirectiveValueBinding)> {
    if vize_carton::is_builtin_directive(directive.name) {
        return None;
    }
    let expression = directive.exp.as_ref()?;
    let variable = directive_binding_name(directive.name);
    let in_setup = bindings.contains_binding(variable.as_str());
    let arg = match directive.arg.as_ref() {
        Some(ExpressionNode::Simple(arg)) if arg.is_static => Some((
            CompactString::new(arg.content),
            (arg.loc.span.start, arg.loc.span.end),
        )),
        _ => None,
    };
    let modifiers = directive
        .modifiers
        .iter()
        .map(|modifier| {
            (
                CompactString::new(modifier.content),
                (modifier.loc.span.start, modifier.loc.span.end),
            )
        })
        .collect();
    let location = expression.loc();
    Some((
        (location.span.start, location.span.end),
        DirectiveValueBinding {
            variable,
            in_setup,
            arg,
            modifiers,
            has_default_alias: false,
            vapor: false,
        },
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
    let variable = binding.variable.as_str();
    if binding.vapor {
        super::vapor_directive::generate(
            ts,
            mappings,
            (name.as_str(), generated_expression, source),
            binding,
            template_offset,
            indent,
        );
        return;
    }
    if binding.in_setup {
        append!(*ts, "{indent}const {name} = __vizeDirective({variable});\n");
    } else {
        let registered = if binding.has_default_alias {
            "typeof __default__"
        } else {
            "unknown"
        };
        append!(
            *ts,
            "{indent}const {name} = __vizeDirective(__vizeRegisteredDirective<{registered}, \"{variable}\">());\n",
        );
    }
    append!(
        *ts,
        "{indent}{name}(null!, {{ ...__vizeDirectiveBindingRest, "
    );
    let mut push_mapped = |ts: &mut String, text: &str, (start, end): (u32, u32)| {
        let gen_start = ts.len();
        ts.push_str(text);
        mappings.push(VizeMapping {
            gen_range: gen_start..ts.len(),
            src_range: (template_offset + start) as usize..(template_offset + end) as usize,
            sub_spans: Vec::new(),
        });
    };
    if let Some((arg, range)) = &binding.arg {
        push_mapped(ts, "arg", *range);
        ts.push_str(": ");
        crate::virtual_ts::helpers::push_ts_string_literal(ts, arg.as_str());
        ts.push_str(", ");
    }
    if let (Some(first), Some(last)) = (binding.modifiers.first(), binding.modifiers.last()) {
        push_mapped(ts, "modifiers", (first.1.0, last.1.1));
        ts.push_str(": { ");
        for (modifier, range) in &binding.modifiers {
            let mut key = String::default();
            crate::virtual_ts::helpers::push_ts_string_literal(&mut key, modifier.as_str());
            push_mapped(ts, key.as_str(), *range);
            ts.push_str(": true, ");
        }
        ts.push_str("}, ");
    }
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
