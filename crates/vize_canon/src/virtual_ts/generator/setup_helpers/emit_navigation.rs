//! Native property identity for statically named calls to the actual macro result.

use std::ops::Range;

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, Program, Statement,
    VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use vize_carton::{CompactString, FxHashSet, String, append};
use vize_croquis::Croquis;

use super::declarations::SetupHelperPlan;
use crate::virtual_ts::{VizeMapping, props::extract_generic_names};

pub(super) struct OwnedEmitCall {
    literal: CompactString,
    content: Range<usize>,
}

/// Public event source roles from the same parsed program and resolved symbols
/// used by generation. The caller must supply the exact program/source pair.
/// This only chooses configured-project extent; it does not authorize edits.
pub fn owned_emit_navigation_source_ranges(
    program: &Program<'_>,
    scoping: &Scoping,
    source: &str,
    summary: &Croquis,
) -> Vec<Range<usize>> {
    if summary.macros.define_emits().is_none()
        || scoping.symbol_ids().any(|symbol| {
            scoping.symbol_scope_id(symbol) == scoping.root_scope_id()
                && scoping.symbol_flags(symbol).is_value()
                && scoping.symbol_name(symbol) == "defineEmits"
        })
    {
        return Vec::new();
    }
    let mut ranges = summary
        .macros
        .emits()
        .iter()
        .filter_map(|event| {
            let (start, end) = summary.macros.emit_declaration(event.name.as_str())?;
            let range = start as usize..end as usize;
            (source.get(range.clone()) == Some(event.name.as_str())).then_some(range)
        })
        .collect::<Vec<_>>();
    ranges.extend(
        collect(program, scoping, source, &FxHashSet::default(), summary)
            .into_iter()
            .map(|call| call.content),
    );
    ranges
}

pub(super) fn collect(
    program: &Program<'_>,
    scoping: &Scoping,
    source: &str,
    shadowed: &FxHashSet<CompactString>,
    summary: &Croquis,
) -> Vec<OwnedEmitCall> {
    let Some(macro_call) = summary.macros.define_emits() else {
        return Vec::new();
    };
    if shadowed.contains("defineEmits") {
        return Vec::new();
    }
    let mut bindings = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                continue;
            };
            if let Some(Expression::CallExpression(call)) = declarator
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
                && call.span.start == macro_call.start
                && call.span.end == macro_call.end
                && let Some(symbol) = binding.symbol_id.get()
                && scoping
                    .get_resolved_references(symbol)
                    .all(|reference| !reference.is_write())
            {
                bindings.push(symbol);
            }
        }
    }
    if bindings.is_empty() {
        return Vec::new();
    }
    let mut collector = EmitCalls {
        bindings,
        scoping,
        source,
        summary,
        calls: Vec::new(),
    };
    collector.visit_program(program);
    collector.calls
}

struct EmitCalls<'s> {
    bindings: Vec<SymbolId>,
    scoping: &'s Scoping,
    source: &'s str,
    summary: &'s Croquis,
    calls: Vec<OwnedEmitCall>,
}

impl<'a> Visit<'a> for EmitCalls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = call.callee.get_inner_expression()
            && let Some(reference) = callee.reference_id.get()
            && let Some(symbol) = self.scoping.get_reference(reference).symbol_id()
            && self.bindings.contains(&symbol)
            && let Some(Argument::StringLiteral(event)) = call.arguments.first()
            && self
                .summary
                .macros
                .emits()
                .iter()
                .any(|definition| definition.name.as_str() == event.value.as_str())
        {
            let content = event.span.start as usize + 1..event.span.end as usize - 1;
            // Escaped source spelling has no byte-exact property projection.
            if !content.is_empty()
                && self.source.get(content.clone()) == Some(event.value.as_str())
                && let Some(literal) = self
                    .source
                    .get(event.span.start as usize..event.span.end as usize)
            {
                self.calls.push(OwnedEmitCall {
                    literal: CompactString::from(literal),
                    content,
                });
            }
        }
        walk::walk_call_expression(self, call);
    }
}

impl SetupHelperPlan {
    pub(crate) fn emit_event_navigation(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        summary: &Croquis,
        generic_param: Option<&str>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        if self.emit_calls.is_empty() {
            return;
        }
        let generic_names = generic_param
            .filter(|_| super::define_emits_runtime_args(summary).is_none())
            .filter(|_| {
                !summary
                    .type_exports
                    .iter()
                    .any(|export| export.name == "Emits")
            })
            .map(extract_generic_names)
            .unwrap_or_default();
        let generic_suffix = if generic_names.is_empty() {
            String::default()
        } else {
            vize_carton::cstr!("<{generic_names}>")
        };
        append!(
            *ts,
            "  const __vize_emit_calls_nav = undefined as unknown as __VizeAuthoredEventMap{generic_suffix};\n"
        );
        for call in &self.emit_calls {
            ts.push_str("  void __vize_emit_calls_nav[");
            let start = ts.len() + 1;
            ts.push_str(&call.literal);
            mappings.push(VizeMapping::new(
                start..start + call.content.len(),
                source_offset(call.content.start)..source_offset(call.content.end),
            ));
            ts.push_str("];\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SetupHelperPlan;
    use vize_carton::String;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    fn plan(source: &str) -> (SetupHelperPlan, vize_croquis::Croquis) {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(source);
        let summary = analyzer.finish();
        (
            SetupHelperPlan::collect(&summary, Some(source), true),
            summary,
        )
    }

    #[test]
    fn event_navigation_uses_the_macro_result_symbol_and_exact_literal_bytes() {
        let source = r#"const fire = defineEmits<{ change: [value: boolean] }>();
fire("change", true);
function owned() { fire('change', true); }
function shadow(fire: (key: string) => void) { fire("change"); }
const unrelated = (key: string) => key;
unrelated("change");
fire("ch\u0061nge", true);
fire("undeclared", true);
const key = "change";
fire(key, true);
"#;
        let (plan, summary) = plan(source);
        let mut generated = String::default();
        let mut mappings = Vec::new();
        plan.emit_event_navigation(&mut generated, &mut mappings, &summary, None, &|offset| {
            offset + 23
        });
        assert_eq!(
            generated.as_str(),
            "  const __vize_emit_calls_nav = undefined as unknown as __VizeAuthoredEventMap;\n  void __vize_emit_calls_nav[\"change\"];\n  void __vize_emit_calls_nav['change'];\n"
        );
        let first_source = source.find("fire(\"change\"").unwrap() + 6;
        let second_source = source.find("fire('change'").unwrap() + 6;
        let first_generated = generated.find("[\"change\"]").unwrap() + 2;
        let second_generated = generated.find("['change']").unwrap() + 2;
        assert_eq!(
            mappings,
            vec![
                crate::virtual_ts::VizeMapping::new(
                    first_generated..first_generated + 6,
                    first_source + 23..first_source + 29,
                ),
                crate::virtual_ts::VizeMapping::new(
                    second_generated..second_generated + 6,
                    second_source + 23..second_source + 29,
                ),
            ]
        );
        let disabled = SetupHelperPlan::collect(&summary, Some(source), false);
        let mut generated = String::default();
        let mut mappings = Vec::new();
        disabled.emit_event_navigation(&mut generated, &mut mappings, &summary, None, &|offset| {
            offset
        });
        assert_eq!((generated.as_str(), mappings), ("", Vec::new()));
    }

    #[test]
    fn mutable_unbound_and_shadowed_macro_calls_have_no_event_projection() {
        for source in [
            "defineEmits<{ change: [] }>(); const emit = (name: string) => name; emit('change');",
            "let emit = defineEmits<{ change: [] }>(); emit('change');",
            "const emit = defineEmits<{ change: [] }>(); emit = (_name: string) => {}; emit('change');",
            "const defineEmits = <T>() => (_name: string) => {}; const emit = defineEmits<{ change: [] }>(); emit('change');",
            "import { defineEmits } from './unrelated'; const emit = defineEmits<{ change: [] }>(); emit('change');",
        ] {
            let (plan, summary) = plan(source);
            let mut generated = String::default();
            let mut mappings = Vec::new();
            plan.emit_event_navigation(&mut generated, &mut mappings, &summary, None, &|offset| {
                offset
            });
            assert_eq!((generated.as_str(), mappings), ("", Vec::new()), "{source}");
        }
    }

    #[test]
    fn tsx_and_generic_events_keep_the_complete_authored_content_range() {
        let source = "const view = <span />; const emit = defineEmits<{ change: [value: T] }>(); emit('change', value);";
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup_with_generic_jsx(source, Some("T"));
        let summary = analyzer.finish();
        let plan = SetupHelperPlan::collect(&summary, Some(source), true);
        let mut generated = String::default();
        let mut mappings = Vec::new();
        plan.emit_event_navigation(
            &mut generated,
            &mut mappings,
            &summary,
            Some("T"),
            &|offset| offset,
        );
        let source_start = source.find("emit('change'").unwrap() + 6;
        let generated_start = generated.find("['change']").unwrap() + 2;
        assert_eq!(
            (generated.as_str(), mappings),
            (
                "  const __vize_emit_calls_nav = undefined as unknown as __VizeAuthoredEventMap<T>;\n  void __vize_emit_calls_nav['change'];\n",
                vec![crate::virtual_ts::VizeMapping::new(
                    generated_start..generated_start + 6,
                    source_start..source_start + 6,
                )],
            )
        );
    }

    #[test]
    fn shared_program_event_roles_keep_exact_ranges_and_exclude_value_and_shadow_calls() {
        let source = "const emit = defineEmits<{ change: [value: boolean] }>(); const change = true; emit(\"change\", change); function shadow(emit: (name: string, value: boolean) => void) { emit(\"change\", change); }";
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup_program(&parsed.program, source, None);
        let summary = analyzer.finish();
        let built = oxc_semantic::SemanticBuilder::new().build(&parsed.program);
        assert!(built.diagnostics.is_empty());
        let declaration = source.find("change:").unwrap();
        let literal = source.find("change\", change").unwrap();
        assert_eq!(
            super::owned_emit_navigation_source_ranges(
                &parsed.program,
                built.semantic.scoping(),
                source,
                &summary,
            ),
            vec![declaration..declaration + 6, literal..literal + 6]
        );
    }

    #[test]
    fn shared_tsx_generic_program_keeps_exact_public_event_roles() {
        let source = "const view = <span />; const emit = defineEmits<{ change: [value: T] }>(); emit('change', value);";
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::tsx()).parse();
        assert!(parsed.diagnostics.is_empty());
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup_program(&parsed.program, source, Some("T"));
        let summary = analyzer.finish();
        let built = oxc_semantic::SemanticBuilder::new().build(&parsed.program);
        assert!(built.diagnostics.is_empty());
        let declaration = source.find("change:").unwrap();
        let literal = source.find("change', value").unwrap();
        assert_eq!(
            super::owned_emit_navigation_source_ranges(
                &parsed.program,
                built.semantic.scoping(),
                source,
                &summary,
            ),
            vec![declaration..declaration + 6, literal..literal + 6]
        );
    }
}
