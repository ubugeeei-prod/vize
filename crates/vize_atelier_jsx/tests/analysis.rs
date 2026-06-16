//! Croquis semantic analysis exposed alongside the lowered roots.

mod common;

use vize_atelier_jsx::{JsxLang, analyze_jsx_program, lower_source, parse_module};
use vize_carton::Bump;
use vize_croquis::BindingMetadata;
use vize_relief::BindingType;

fn sorted_binding_entries(bindings: &BindingMetadata) -> Vec<(String, BindingType)> {
    let mut entries: Vec<_> = bindings
        .iter()
        .map(|(name, binding_type)| (name.to_owned(), binding_type))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

fn sorted_bindings(out: &vize_atelier_jsx::LowerOutput<'_>) -> Vec<(String, BindingType)> {
    sorted_binding_entries(out.bindings())
}

#[test]
fn top_level_bindings_are_collected() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const count = 1;\nconst App = () => <div>{count}</div>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert_eq!(
        sorted_bindings(&out),
        vec![
            ("App".to_owned(), BindingType::SetupConst),
            ("count".to_owned(), BindingType::LiteralConst)
        ]
    );
}

#[test]
fn ref_binding_is_recognized_as_reactive() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "import { ref } from 'vue';\nconst n = ref(0);\nconst C = () => <p>{n.value}</p>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.bindings().is_ref("n"));
}

#[test]
fn analysis_runs_on_tsx_modules() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const label: string = 'x';\nconst C = (): JSX.Element => <span>{label}</span>;",
        JsxLang::Tsx,
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert_eq!(
        sorted_bindings(&out),
        vec![
            ("C".to_owned(), BindingType::SetupConst),
            ("label".to_owned(), BindingType::LiteralConst)
        ]
    );
}

#[test]
fn undefined_binding_is_absent() {
    let bump = Bump::new();
    let out = lower_source(&bump, "const C = () => <div>{ghost}</div>;", JsxLang::Jsx);
    assert_eq!(
        sorted_bindings(&out),
        vec![("C".to_owned(), BindingType::SetupConst)]
    );
}

#[test]
fn parse_free_croquis_analysis_matches_lowering_analysis() {
    let source = r#"
        import { ref } from 'vue';
        const count = ref(0);
        const props = defineProps<{ label: string }>();
        const C = (): JSX.Element => <span>{props.label}{count.value}</span>;
    "#;

    let allocator = oxc_allocator::Allocator::default();
    let parsed = parse_module(&allocator, source, JsxLang::Tsx);
    assert!(
        parsed.diagnostics.is_empty(),
        "parse diagnostics: {:?}",
        parsed.diagnostics
    );

    let analyzed = analyze_jsx_program(&parsed.program, source);
    let bump = Bump::new();
    let lowered = lower_source(&bump, source, JsxLang::Tsx);
    assert!(!lowered.has_errors(), "{:?}", lowered.diagnostics);

    assert!(analyzed.bindings.is_ref("count"));
    assert!(analyzed.bindings.contains("props"));
    assert_eq!(
        sorted_binding_entries(&analyzed.bindings),
        sorted_bindings(&lowered)
    );
}
