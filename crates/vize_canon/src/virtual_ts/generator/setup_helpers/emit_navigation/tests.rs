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
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
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
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::tsx()).parse();
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
