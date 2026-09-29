#[path = "reactivity_loss_spreads.rs"]
mod spreads;
use spreads::{computed_value_spread_spans, diagnostic_key, reactivity_loss_diagnostic};

use super::document::TypeAwareDocument;
use super::{
    LintResult, Linter, RULE_NO_REACTIVITY_LOSS, markers::marker_insert_offset, push_warning,
};
use crate::diagnostic::LintDiagnostic;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, ChainElement, Expression,
    Function, FunctionBody, ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName,
    ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind, SpreadElement,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_call_expression, walk_function,
        walk_import_declaration, walk_spread_element, walk_variable_declarator,
    },
};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_croquis::{
    reactivity::{ReactivityLoss, ReactivityLossKind},
    script_parser::ScriptParseResult,
};
use vize_l0::{CompactString, FxHashSet, String, ToCompactString, profile};

#[derive(Clone)]
pub(super) struct ReactivityLossQuery {
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
    message: CompactString,
    help: &'static str,
}

impl ReactivityLossQuery {
    #[inline]
    pub fn owner_key(&self) -> u64 {
        ((self.source_start as u64) << 32) | self.source_end as u64
    }

    pub fn diagnostic(&self, script_offset: u32) -> LintDiagnostic {
        LintDiagnostic::warn(
            RULE_NO_REACTIVITY_LOSS,
            self.message.clone(),
            script_offset + self.source_start,
            script_offset + self.source_end,
        )
        .with_help(self.help)
    }
}

pub(super) fn collect_reactivity_loss_queries(
    linter: &Linter,
    result: &mut LintResult,
    parse_result: &ScriptParseResult,
    script_content: &str,
    script_offset: u32,
    virtual_ts: &mut TypeAwareDocument,
) -> Vec<ReactivityLossQuery> {
    if !(linter.registry.has_rule(RULE_NO_REACTIVITY_LOSS)
        && linter.is_rule_enabled(RULE_NO_REACTIVITY_LOSS))
    {
        return Vec::new();
    }
    if !parse_result.reactivity.has_losses() {
        return Vec::new();
    }

    let mut queries = Vec::with_capacity(parse_result.reactivity.losses().len());
    let mut immediate = FxHashSet::default();
    let exempt_value_spreads = if parse_result
        .reactivity
        .losses()
        .iter()
        .any(|loss| matches!(loss.kind, ReactivityLossKind::ReactiveSpread { .. }))
    {
        computed_value_spread_spans(script_content)
    } else {
        Default::default()
    };

    for loss in parse_result.reactivity.losses() {
        if matches!(loss.kind, ReactivityLossKind::ReactiveSpread { .. })
            && exempt_value_spreads.contains(&(loss.start, loss.end))
        {
            continue;
        }
        let diagnostic = reactivity_loss_diagnostic(loss);
        let expressions = query_expressions_for_loss(loss, script_content);

        if expressions.is_empty() {
            let key = diagnostic_key(loss.start, loss.end);
            if immediate.insert(key) {
                push_warning(result, diagnostic.diagnostic(script_offset));
            }
            continue;
        }

        for expression in expressions {
            if let Some(query) =
                push_reactivity_loss_marker(virtual_ts, expression.as_str(), &diagnostic)
            {
                queries.push(query);
            }
        }
    }

    queries
}

fn push_reactivity_loss_marker(
    virtual_ts: &mut TypeAwareDocument,
    expression_source: &str,
    diagnostic: &ReactivityLossQuery,
) -> Option<ReactivityLossQuery> {
    let expression_source = expression_source.trim();
    if expression_source.is_empty() {
        return None;
    }
    let insert_offset = marker_insert_offset(&virtual_ts.content)?;

    let mut marker_name = String::with_capacity(32);
    marker_name.push_str("__vize_patina_reactivity_");
    marker_name.push_str(diagnostic.source_start.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(diagnostic.source_end.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(virtual_ts.content.len().to_compact_string().as_str());

    let mut line = String::with_capacity(marker_name.len() + expression_source.len() + 24);
    line.push_str("    const ");
    let name_offset = line.len() as u32;
    line.push_str(&marker_name);
    line.push_str(" = (");
    line.push_str(expression_source);
    line.push_str(");\n");

    let mut query = diagnostic.clone();
    query.generated_offset = insert_offset as u32 + name_offset;
    virtual_ts.content.insert_str(insert_offset, &line);
    Some(query)
}

fn query_expressions_for_loss(loss: &ReactivityLoss, script_content: &str) -> Vec<CompactString> {
    match &loss.kind {
        ReactivityLossKind::PropsDestructure { .. } => Vec::new(),
        ReactivityLossKind::RefValueExtract { .. }
        | ReactivityLossKind::ReactivePropertyExtract { .. }
        | ReactivityLossKind::FunctionArgumentExtract { .. }
        | ReactivityLossKind::GetterCallExtract { .. }
        | ReactivityLossKind::PlainValueAlias { .. } => script_content
            .get(loss.start as usize..loss.end as usize)
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .map(|source| vec![CompactString::new(source)])
            .unwrap_or_default(),
        ReactivityLossKind::ReactiveDestructure { .. }
        | ReactivityLossKind::RefValueDestructure { .. }
        | ReactivityLossKind::ReactiveSpread { .. }
        | ReactivityLossKind::ReactiveReassign { .. } => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::lint_sfc_with_corsa;
    use crate::{LintPreset, Linter};

    fn spread_messages(source: &str) -> Vec<std::string::String> {
        let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
        let wrapped = format!("<script setup lang=\"ts\">\n{source}\n</script>\n");
        let result = lint_sfc_with_corsa(&linter, &wrapped, "Fixture.vue");
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "type/no-reactivity-loss")
            .map(|diagnostic| diagnostic.message.as_str().to_string())
            .collect()
    }

    #[test]
    fn value_spread_inside_computed_getter_is_not_reactivity_loss() {
        let messages = spread_messages(
            r#"
import { computed as useComputed, ref } from 'vue'
const state = ref({ count: 1, tags: ['a'] })
const alias = useComputed
const view = computed(() => ({ ...state.value }))
const block = computed(() => { return { ...state.value } })
const options = computed({ get() { return { ...state.value } } })
const viaAlias = alias(() => ({ ...state.value }))
const outside = { ...state.value }
const nested = computed(() => {
  const leak = () => ({ ...state.value })
  return leak()
})
const tags = { ...state.value.tags }
"#,
        );
        let mut value_spreads = 0;
        let mut tag_spreads = 0;
        for message in &messages {
            if message.contains("Spreading 'state.value.tags'") {
                tag_spreads += 1;
            } else if message.contains("Spreading 'state.value'") {
                value_spreads += 1;
            }
        }
        assert_eq!(value_spreads, 2, "{messages:?}");
        assert_eq!(tag_spreads, 1, "{messages:?}");
    }
}
