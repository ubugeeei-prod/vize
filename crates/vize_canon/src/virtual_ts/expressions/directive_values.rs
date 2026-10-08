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

mod presence;
pub(crate) use presence::generate_valueless_directive_presence;

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
    /// Template span of the directive name (`v-focus`), not its value.
    name_range: (u32, u32),
    /// A Vapor component calls the directive itself, with a value getter,
    /// instead of a hook that receives a binding object.
    vapor: bool,
    has_value: bool,
}

pub(crate) fn collect_directive_value_bindings(
    root: Option<&RootNode<'_>>,
    bindings: ScriptBindings<'_>,
    (has_default_alias, vapor): (bool, bool),
    (enabled, include_valueless): (bool, bool),
) -> DirectiveValueBindings {
    let mut collected = DirectiveValueBindings::default();
    let Some(root) = root.filter(|_| enabled) else {
        return collected;
    };
    for child in &root.children {
        collect_child_bindings(child, bindings, &mut collected, include_valueless);
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
    include_valueless: bool,
) {
    match child {
        TemplateChildNode::Element(element) => {
            collect_element_bindings(element, bindings, collected, include_valueless)
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect_child_bindings(child, bindings, collected, include_valueless);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect_child_bindings(child, bindings, collected, include_valueless);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect_child_bindings(child, bindings, collected, include_valueless);
            }
        }
        _ => {}
    }
}

fn collect_element_bindings(
    element: &ElementNode<'_>,
    bindings: ScriptBindings<'_>,
    collected: &mut DirectiveValueBindings,
    include_valueless: bool,
) {
    for prop in &element.props {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        if let Some((range, binding)) =
            directive_value_binding(directive, bindings, include_valueless)
        {
            collected.insert(range, binding);
        }
    }
    for child in &element.children {
        collect_child_bindings(child, bindings, collected, include_valueless);
    }
}

fn directive_value_binding(
    directive: &DirectiveNode<'_>,
    bindings: ScriptBindings<'_>,
    include_valueless: bool,
) -> Option<((u32, u32), DirectiveValueBinding)> {
    if vize_carton::is_builtin_directive(directive.name) {
        return None;
    }
    let expression = directive
        .exp
        .as_ref()
        .filter(|expression| match expression {
            ExpressionNode::Simple(expression) => !expression.content.trim().is_empty(),
            _ => true,
        });
    if expression.is_none() && !include_valueless {
        return None;
    }
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
    let name_start = directive.loc.span.start;
    let raw_len = directive
        .raw_name
        .map(|raw| raw.len())
        .unwrap_or(directive.name.len().saturating_add(2));
    let name_end = name_start
        .saturating_add(u32::try_from(raw_len).unwrap_or(u32::MAX))
        .min(directive.loc.span.end);
    Some((
        expression.map_or((name_start, name_end), |expression| {
            let location = expression.loc();
            (location.span.start, location.span.end)
        }),
        DirectiveValueBinding {
            variable,
            in_setup,
            arg,
            modifiers,
            has_default_alias: false,
            vapor: false,
            name_range: (name_start, name_end),
            has_value: expression.is_some(),
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
    (generated_expression, mapped_start, mapped_len): (&str, usize, usize),
    template_offset: u32,
    (indent, check_unknown_directives): (&str, bool),
) {
    emit_unknown_directive_presence(
        ts,
        mappings,
        binding,
        template_offset,
        indent,
        check_unknown_directives,
    );
    let source = (template_offset + expr.start) as usize..(template_offset + expr.end) as usize;
    let name = vize_carton::cstr!("__vize_directive_check_{}", expr.start);
    let variable = binding.variable.as_str();
    if binding.vapor {
        super::vapor_directive::generate(
            ts,
            mappings,
            (
                name.as_str(),
                generated_expression,
                (mapped_start, mapped_len),
                source,
            ),
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
    let value_end = value_start + mapped_start + mapped_len;
    mappings.push(VizeMapping {
        gen_range: value_start..value_end,
        src_range: source,
        sub_spans: Vec::new(),
    });
    append!(
        *ts,
        ") }}, ...__vizeDirectiveTail({name})); // CustomDirective\n"
    );
}

/// A missing registry entry is `TS2339` on the directive name. The shared
/// `__vizeRegisteredDirective` fallback stays `unknown` so a `vue` package
/// without `GlobalDirectives` does not error on the value check itself.
fn emit_unknown_directive_presence(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    binding: &DirectiveValueBinding,
    template_offset: u32,
    indent: &str,
    enabled: bool,
) {
    if !enabled || binding.in_setup {
        return;
    }
    let (start, end) = binding.name_range;
    if start >= end {
        return;
    }
    let variable = binding.variable.as_str();
    let registered = if binding.has_default_alias {
        "typeof __default__"
    } else {
        "unknown"
    };
    let local = vize_carton::cstr!("__vize_unknown_directive_{start}");
    append!(*ts, "{indent}const {{ ");
    let gen_start = ts.len();
    crate::virtual_ts::helpers::push_ts_string_literal(ts, variable);
    let gen_end = ts.len();
    append!(*ts, ": {local} }} = {{}} as (");
    let push_key = |ts: &mut String| {
        crate::virtual_ts::helpers::push_ts_string_literal(ts, variable);
    };
    push_key(ts);
    append!(
        *ts,
        " extends keyof __VizeLocalDirectives<{registered}> ? {{ "
    );
    push_key(ts);
    append!(*ts, ": unknown }} : ");
    push_key(ts);
    append!(*ts, " extends keyof __VizeGlobalDirectives ? {{ ");
    push_key(ts);
    append!(*ts, ": unknown }} : {{}});\n{indent}void {local};\n");
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: (template_offset + start) as usize..(template_offset + end) as usize,
        sub_spans: Vec::new(),
    });
}

#[cfg(test)]
#[path = "directive_values/naming_tests.rs"]
mod tests;
