//! Conservative template reads used by setup binding anchors.

use vize_atelier_sfc::script::resolve_template_read_identifiers;
use vize_carton::{FxHashSet, String};
use vize_croquis::{Croquis, ScopeData};

pub(in crate::virtual_ts::generator) fn template_usage(
    summary: &Croquis,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    generation_options: crate::virtual_ts::types::VirtualTsGenerationOptions<'_>,
) -> (FxHashSet<String>, bool) {
    let names = collect_template_referenced_names(
        summary,
        template_ast,
        generation_options.extra_template_referenced_names,
    );
    let has_scope = template_ast.is_some() || !names.is_empty();
    (names, has_scope)
}

fn collect_template_referenced_names(
    summary: &Croquis,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    extra_template_referenced_names: Option<&FxHashSet<String>>,
) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let mut expressions = FxHashSet::default();

    if let Some(template_ast) = template_ast {
        names.extend(resolve_template_read_identifiers(template_ast));
    }

    if let Some(extra_names) = extra_template_referenced_names {
        names.extend(extra_names.iter().cloned());
    }

    for expression in &summary.template_expressions {
        collect_expression_identifiers(&mut names, &mut expressions, expression.content.as_str());
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_expression_identifiers(&mut names, &mut expressions, guard.as_str());
        }
    }

    for usage in vize_croquis::facts::component_usage_list(summary) {
        names.insert(usage.name.as_str().into());
        if let Some(guard) = usage.vif_guard.as_ref() {
            collect_expression_identifiers(&mut names, &mut expressions, guard.as_str());
        }
        for prop in &usage.props {
            if prop.is_dynamic
                && let Some(value) = prop.value.as_ref()
            {
                collect_expression_identifiers(&mut names, &mut expressions, value.as_str());
            }
        }
        for event in &usage.events {
            if let Some(handler) = event.handler.as_ref() {
                collect_expression_identifiers(&mut names, &mut expressions, handler.as_str());
            }
        }
    }

    for component in vize_croquis::facts::used_component_name_list(summary) {
        names.insert(component.as_str().into());
    }

    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VFor(data) => {
                collect_pattern_default_identifiers(&mut names, data.value_alias.as_str());
                collect_expression_identifiers(&mut names, &mut expressions, data.source.as_str());
                if let Some(key_expression) = data.key_expression.as_ref() {
                    collect_expression_identifiers(
                        &mut names,
                        &mut expressions,
                        key_expression.as_str(),
                    );
                }
            }
            ScopeData::VSlot(data) => {
                if let Some(pattern) = data.props_pattern.as_ref() {
                    collect_pattern_default_identifiers(&mut names, pattern.as_str());
                }
            }
            ScopeData::EventHandler(data) => {
                if let Some(handler) = data.handler_expression.as_ref() {
                    collect_expression_identifiers(&mut names, &mut expressions, handler.as_str());
                }
            }
            _ => {}
        }
    }

    names
}

/// A destructuring default (`{ val = fallback }`) is a template read of the
/// binding it names: without it the default keeps the setup binding's raw
/// `Ref` type instead of the unwrapped value every other template read sees.
/// A pattern without `=` reads nothing.
fn collect_pattern_default_identifiers(names: &mut FxHashSet<String>, pattern: &str) {
    if !pattern.contains('=') {
        return;
    }
    // The assignment form of the pattern: the identifier walk reads a
    // destructuring target's defaults, not an arrow parameter's.
    let assignment = vize_carton::cstr!("({pattern} = 0)");
    for identifier in vize_croquis::drawer::extract_identifiers_oxc(assignment.as_str()) {
        names.insert(identifier.as_str().into());
    }
}

fn collect_expression_identifiers(
    names: &mut FxHashSet<String>,
    expressions: &mut FxHashSet<String>,
    expression: &str,
) {
    // Croquis records a source expression in multiple semantic views. This is
    // one conservative name union, so identical text only needs parsing once.
    if !expressions.insert(expression.into()) {
        return;
    }
    for identifier in vize_croquis::drawer::extract_identifiers_oxc(expression) {
        names.insert(identifier.as_str().into());
    }
}
