//! Conservative template reads used by setup binding anchors.

use vize_atelier_sfc::script::resolve_template_read_identifiers;
use vize_carton::{FxHashSet, String, camelize, capitalize};
use vize_croquis::{Croquis, ScopeData};

pub(in crate::virtual_ts::generator) fn template_usage(
    summary: &Croquis,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    generation_options: crate::virtual_ts::types::VirtualTsGenerationOptions<'_>,
) -> (FxHashSet<String>, FxHashSet<String>, bool) {
    let (names, value_names) = collect_template_referenced_names(
        summary,
        template_ast,
        generation_options.extra_template_referenced_names,
    );
    let has_scope = template_ast.is_some() || !names.is_empty();
    (names, value_names, has_scope)
}

fn collect_template_referenced_names(
    summary: &Croquis,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    extra_template_referenced_names: Option<&FxHashSet<String>>,
) -> (FxHashSet<String>, FxHashSet<String>) {
    let mut names = FxHashSet::default();
    // Keep broad TypeScript references for authored unused-binding behavior,
    // but only runtime reads may drive synthetic value captures and anchors.
    let mut value_names = summary
        .template_expression_reads
        .clone()
        .unwrap_or_default();
    let mut expressions = FxHashSet::default();

    if let Some(template_ast) = template_ast {
        names.extend(resolve_template_read_identifiers(template_ast));
    }

    if let Some(extra_names) = extra_template_referenced_names {
        names.extend(extra_names.iter().cloned());
        value_names.extend(extra_names.iter().cloned());
    }

    // A setup-local custom directive is a runtime value even when it has no
    // bound expression: `<input v-focus>` resolves the imported `vFocus`.
    // Keep that read in the value set so the unused-binding anchor does not
    // discard an import whose only consumer is the directive name (#3140).
    for directive in &summary.used_directives {
        let camel = camelize(directive.as_str());
        let capitalized = capitalize(camel.as_str());
        let mut binding = String::with_capacity(capitalized.len() + 1);
        binding.push('v');
        binding.push_str(capitalized.as_str());
        value_names.insert(binding);
    }

    if summary.template_expression_reads.is_none() {
        collect_legacy_expression_identifiers(summary, &mut value_names, &mut expressions);
    }

    for usage in vize_croquis::facts::component_usage_slice(summary) {
        names.insert(usage.name.as_str().into());
        value_names.insert(usage.name.as_str().into());
    }

    for component in vize_croquis::facts::used_component_name_list(summary) {
        names.insert(component.as_str().into());
        value_names.insert(component.as_str().into());
    }

    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VFor(data) => {
                collect_pattern_default_identifiers(&mut value_names, data.value_alias.as_str());
            }
            ScopeData::VSlot(data) => {
                if let Some(pattern) = data.props_pattern.as_ref() {
                    collect_pattern_default_identifiers(&mut value_names, pattern.as_str());
                }
            }
            _ => {}
        }
    }

    names.extend(value_names.iter().cloned());
    (names, value_names)
}

fn collect_legacy_expression_identifiers(
    summary: &Croquis,
    names: &mut FxHashSet<String>,
    expressions: &mut FxHashSet<String>,
) {
    for expression in &summary.template_expressions {
        collect_expression_identifiers(names, expressions, expression.content.as_str());
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_expression_identifiers(names, expressions, guard.as_str());
        }
    }

    for usage in vize_croquis::facts::component_usage_slice(summary) {
        if let Some(guard) = usage.vif_guard.as_ref() {
            collect_expression_identifiers(names, expressions, guard.as_str());
        }
        for prop in &usage.props {
            if prop.is_dynamic
                && let Some(value) = prop.value.as_ref()
            {
                collect_expression_identifiers(names, expressions, value.as_str());
            }
        }
        for event in &usage.events {
            if let Some(handler) = event.handler.as_ref() {
                collect_expression_identifiers(names, expressions, handler.as_str());
            }
        }
    }

    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VFor(data) => {
                collect_expression_identifiers(names, expressions, data.source.as_str());
                if let Some(key_expression) = data.key_expression.as_ref() {
                    collect_expression_identifiers(names, expressions, key_expression.as_str());
                }
            }
            ScopeData::EventHandler(data) => {
                if let Some(handler) = data.handler_expression.as_ref() {
                    collect_expression_identifiers(names, expressions, handler.as_str());
                }
            }
            _ => {}
        }
    }
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

#[cfg(test)]
mod tests {
    use super::collect_template_referenced_names;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    #[test]
    fn retained_expression_reads_match_the_legacy_reparse_union() {
        let script = r#"
import Card from './Card.vue'
const show = true
const rows = [{ id: 1 }]
const field = 'value'
const fallback = { name: 'fallback' }
const click = (value: string) => value
const format = (row: { id: number }, index: number) => `${row.id}:${index}`
const external = 'outside'
const vFocus = {}
"#;
        let template = r#"
<Card v-if="show && rows.length" :value="field" @click="() => click(field)" v-focus>
  <template #default="{ item = fallback }">
    <span v-for="(row, index) in rows" :key="row.id" :title="format(row, index)">
      {{ item.name }} {{ external }}
    </span>
  </template>
</Card>
"#;
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let mut summary = analyzer.finish();

        assert!(summary.template_expression_reads.is_some());
        let retained = collect_template_referenced_names(&summary, Some(&root), None);
        summary.template_expression_reads = None;
        let legacy = collect_template_referenced_names(&summary, Some(&root), None);

        assert_eq!(retained, legacy);
        for expected in [
            "show", "rows", "field", "click", "fallback", "row", "index", "format", "item",
            "external", "Card", "vFocus",
        ] {
            assert!(retained.1.contains(expected), "missing {expected}");
        }
    }
}
