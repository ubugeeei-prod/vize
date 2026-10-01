//! Opt-in strict boolean conditions share the existing native checker session.
use super::{
    CorsaTypeAwareSession, LintResult, RULE_STRICT_BOOLEAN, document::TypeAwareDocument,
    push_warning,
};
use crate::diagnostic::LintDiagnostic;
use corsa::lint::{LintNode, TextRange, run_default_type_aware_rule_owned};
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::json;
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::{String, config::StrictBooleanExpressionsOptions};
use vize_relief::RootNode;

mod queries;
mod template;
#[cfg(test)]
mod tests;

pub(super) struct Plan {
    conditions: Vec<queries::Condition>,
    offsets: Vec<u32>,
    targets: Vec<u32>,
}

impl Plan {
    pub(super) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

pub(super) fn plan(
    descriptor: &SfcDescriptor<'_>,
    template: Option<(&RootNode<'_>, u32)>,
    document: &mut TypeAwareDocument,
) -> Plan {
    let mut conditions = Vec::new();
    for block in descriptor
        .script
        .iter()
        .chain(descriptor.script_setup.iter())
    {
        let allocator = Allocator::default();
        let source_type = if matches!(block.lang.as_deref(), Some("tsx" | "jsx")) {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        let parsed = Parser::new(&allocator, &block.content, source_type).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            continue;
        }
        let mut collector = queries::Collector {
            source: &block.content,
            conditions: Vec::new(),
        };
        collector.visit_program(&parsed.program);
        let origin = block.loc.start as u32;
        conditions.extend(
            collector
                .conditions
                .into_iter()
                .map(|condition| queries::Condition {
                    start: origin + condition.start,
                    end: origin + condition.end,
                    anchor: origin + condition.anchor,
                }),
        );
    }
    if let Some((root, offset)) = template {
        template::collect(root, offset, &mut conditions);
    }
    conditions.sort_unstable_by_key(|condition| (condition.start, condition.end));
    conditions.dedup_by_key(|condition| (condition.start, condition.end));
    let mut offsets = Vec::new();
    conditions.retain(
        |condition| match document.generated_offset(condition.anchor) {
            Some(offset) => {
                offsets.push(offset);
                true
            }
            None => false,
        },
    );
    let mut targets = Vec::new();
    if !conditions.is_empty() {
        let suffix = document.content.len();
        document.content.push('\n');
        for (index, ty) in [
            "boolean",
            "string",
            "number",
            "bigint",
            "object | symbol",
            "false",
            "\"\"",
            "0",
            "0n",
        ]
        .iter()
        .enumerate()
        {
            let name = vize_l0::cstr!("__vize_boolean_target_{suffix}_{index}");
            document.content.push_str("declare const ");
            targets.push(document.content.len() as u32);
            document.content.push_str(&name);
            document.content.push_str(": ");
            document.content.push_str(ty);
            document.content.push_str(";\n");
        }
    }
    Plan {
        conditions,
        offsets,
        targets,
    }
}

pub(super) fn evaluate(
    session: &CorsaTypeAwareSession,
    source: &str,
    plan: &Plan,
    options: StrictBooleanExpressionsOptions,
    result: &mut LintResult,
) -> Result<(), String> {
    if plan.is_empty() {
        return Ok(());
    }
    let parts = session.boolean_type_parts(source, &plan.targets, &plan.offsets)?;
    for (condition, parts) in plan.conditions.iter().zip(parts) {
        let Some(parts) = parts else {
            continue;
        };
        let range = TextRange::new(condition.start, condition.end);
        let mut test = node("Identifier", range);
        test.fields
            .insert("__conditionTypeParts".into(), json!(parts));
        let mut parent = node("IfStatement", range);
        parent.children.insert("test".into(), test);
        parent
            .fields
            .insert("__ruleOptions".into(), json!([options]));
        for diagnostic in run_default_type_aware_rule_owned("strict-boolean-expressions", parent)
            .unwrap_or_default()
        {
            push_warning(
                result,
                LintDiagnostic::warn(
                    RULE_STRICT_BOOLEAN,
                    diagnostic.message,
                    diagnostic.range.start,
                    diagnostic.range.end,
                ),
            );
        }
    }
    Ok(())
}

fn node(kind: &str, range: TextRange) -> LintNode {
    LintNode {
        kind: kind.into(),
        range,
        text: None,
        type_texts: Vec::new(),
        property_names: Vec::new(),
        fields: Default::default(),
        children: Default::default(),
        child_lists: Default::default(),
    }
}
