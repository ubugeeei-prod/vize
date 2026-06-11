//! Span collection/merging, template-referenced-name discovery, and
//! `export default` rewriting used while assembling the virtual TypeScript.

use vize_croquis::{Croquis, ScopeData};

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;

pub(super) fn collect_template_referenced_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();

    for expression in &summary.template_expressions {
        collect_expression_identifiers(&mut names, expression.content.as_str());
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_expression_identifiers(&mut names, guard.as_str());
        }
    }

    for usage in &summary.component_usages {
        names.insert(usage.name.as_str().into());
        if let Some(guard) = usage.vif_guard.as_ref() {
            collect_expression_identifiers(&mut names, guard.as_str());
        }
        for prop in &usage.props {
            if prop.is_dynamic
                && let Some(value) = prop.value.as_ref()
            {
                collect_expression_identifiers(&mut names, value.as_str());
            }
        }
        for event in &usage.events {
            if let Some(handler) = event.handler.as_ref() {
                collect_expression_identifiers(&mut names, handler.as_str());
            }
        }
    }

    for component in &summary.used_components {
        names.insert(component.as_str().into());
    }

    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VFor(data) => {
                collect_expression_identifiers(&mut names, data.source.as_str());
                if let Some(key_expression) = data.key_expression.as_ref() {
                    collect_expression_identifiers(&mut names, key_expression.as_str());
                }
            }
            ScopeData::EventHandler(data) => {
                if let Some(handler) = data.handler_expression.as_ref() {
                    collect_expression_identifiers(&mut names, handler.as_str());
                }
            }
            _ => {}
        }
    }

    names
}

fn collect_expression_identifiers(names: &mut FxHashSet<String>, expression: &str) {
    for identifier in vize_croquis::analyzer::extract_identifiers_oxc(expression) {
        names.insert(identifier.as_str().into());
    }
}

pub(super) fn merge_overlapping_spans(mut spans: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    spans.retain(|(start, end)| start < end);
    spans.sort_by_key(|&(start, end)| (start, end));

    let mut merged: Vec<(u32, u32)> = Vec::with_capacity(spans.len());
    for (start, end) in spans {
        if let Some((_, previous_end)) = merged.last_mut()
            && start <= *previous_end
        {
            *previous_end = (*previous_end).max(end);
            continue;
        }
        merged.push((start, end));
    }
    merged
}

/// Generated identifier resolving Vue's `defineComponent` (see
/// `DEFINE_COMPONENT_HELPER`). Plain object-literal default exports are
/// wrapped in a call to it so TypeScript binds `this` inside
/// computed/methods via Vue's `ThisType` machinery instead of the bare
/// object literal (which produced TS2339 false positives).
pub(super) const DEFINE_COMPONENT_REF: &str = "__vizeDefineComponent";

/// Module-scope declaration for `DEFINE_COMPONENT_REF`. Uses the same
/// `import('vue')` reference form as the other generated vue helpers so it
/// never collides with user imports.
pub(super) const DEFINE_COMPONENT_HELPER: &str =
    "declare const __vizeDefineComponent: typeof import('vue').defineComponent;\n";

pub(super) fn rewrite_export_default_for_module_scope(
    text: &str,
    default_object: Option<(usize, usize, usize)>,
) -> String {
    // Plain `export default { ... }` (the Options API shape) is wrapped with
    // Vue's `defineComponent`; any other default-export shape falls through
    // to the line-based `const __default__ =` rewrite below.
    if let Some(output) = wrap_default_export_object(text, default_object) {
        return output;
    }

    let mut output = String::with_capacity(text.len());
    for segment in text.split_inclusive('\n') {
        let (line_with_optional_cr, newline) = segment
            .strip_suffix('\n')
            .map_or((segment, ""), |line| (line, "\n"));
        let (line, carriage_return) = line_with_optional_cr
            .strip_suffix('\r')
            .map_or((line_with_optional_cr, ""), |line| (line, "\r"));

        let trimmed_line = line.trim_start();
        if let Some(default_expr) = trimmed_line
            .strip_prefix("export default")
            .filter(|rest| rest.chars().next().is_none_or(char::is_whitespace))
        {
            let leading_ws = &line[..line.len() - trimmed_line.len()];
            append!(output, "{leading_ws}const __default__ ={default_expr}");
        } else {
            output.push_str(line);
        }
        output.push_str(carriage_return);
        output.push_str(newline);
    }

    output
}

/// Rewrite `export default { ... }` to
/// `const __default__ = __vizeDefineComponent({ ... })` using the
/// `(export_start, object_start, object_end)` offsets (relative to `text`)
/// located by the AST. Returns `None` when the offsets do not describe a
/// plain object-literal default export inside `text`.
fn wrap_default_export_object(
    text: &str,
    default_object: Option<(usize, usize, usize)>,
) -> Option<String> {
    const EXPORT_DEFAULT: &str = "export default";

    let (export_start, object_start, object_end) = default_object?;
    if object_end > text.len() || object_start >= object_end {
        return None;
    }
    let keyword_end = export_start.checked_add(EXPORT_DEFAULT.len())?;
    if keyword_end > object_start
        || !text.is_char_boundary(export_start)
        || !text.is_char_boundary(object_start)
        || !text.is_char_boundary(object_end)
        || !text[export_start..].starts_with(EXPORT_DEFAULT)
        || !text[object_start..].starts_with('{')
    {
        return None;
    }

    let mut output =
        String::with_capacity(text.len() + DEFINE_COMPONENT_REF.len() + EXPORT_DEFAULT.len());
    output.push_str(&text[..export_start]);
    output.push_str("const __default__ =");
    output.push_str(&text[keyword_end..object_start]);
    output.push_str(DEFINE_COMPONENT_REF);
    output.push('(');
    output.push_str(&text[object_start..object_end]);
    output.push(')');
    output.push_str(&text[object_end..]);
    Some(output)
}

pub(super) fn is_local_setup_binding(summary: &Croquis, name: &str) -> bool {
    let Some(&(start, end)) = summary.binding_spans.get(name) else {
        return true;
    };

    !summary
        .import_statements
        .iter()
        .any(|import| start >= import.start && end <= import.end)
}
