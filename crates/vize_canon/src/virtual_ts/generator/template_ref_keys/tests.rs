#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::TemplateRefUsage;
use crate::virtual_ts::generator::setup_helpers::SetupHelperPlan;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::String;

fn usage(script: &str) -> TemplateRefUsage {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let built = SemanticBuilder::new().build(&parsed.program);
    TemplateRefUsage::collect(&parsed.program, built.semantic.scoping(), script)
}

fn generate(script: &str) -> String {
    let source = format!("import {{ useTemplateRef }} from 'vue';\n{script}");
    let usage = usage(&source);
    let script = source.as_str();
    let mut checks = usage.key_checks(true);
    let mut ts = String::default();
    let mut mappings = Vec::new();
    let mut start = 0;
    for line in script.split_inclusive('\n') {
        let end = start + line.len();
        checks.emit_for_line(&mut ts, &mut mappings, (start, end), &|offset| offset);
        let mut line = std::borrow::Cow::Borrowed(line);
        checks.specialize_line(&mut line, start, ts.len(), &|offset| offset);
        ts.push_str(&line);
        start = end;
    }
    for mapping in mappings {
        assert_eq!(
            ts[mapping.gen_range].replace('"', "'"),
            script[mapping.src_range]
        );
    }
    ts.strip_prefix("import { useTemplateRef } from 'vue';\n")
        .unwrap()
        .into()
}

#[test]
fn only_owned_untyped_calls_outside_functions_infer_template_refs() {
    let script = "const a = useTemplateRef('a')\nfunction f() { useTemplateRef('inner') }\nconst g = () => useTemplateRef('arrow')\nuseTemplateRef(name)\nuseTemplateRef<HTMLElement>('typed')\n";
    let plan = usage(script);
    assert!(plan.needs_registry());
    let keys: Vec<_> = plan
        .calls
        .iter()
        .filter_map(|call| call.literal.as_ref())
        .map(|(literal, key)| (&script[literal.clone()], key.as_str()))
        .collect();
    assert_eq!(keys, [("'a'", "a")]);
    assert!(plan.key_checks(false).calls.is_empty());
    assert!(!usage("useTemplateRef<HTMLDivElement, 'custom'>('custom')").needs_registry());
    assert!(!usage("function nested() { return useTemplateRef('box') }").needs_registry());
}

#[test]
fn multiline_calls_keep_key_checks_at_the_statement_and_preserve_typed_calls() {
    assert_eq!(
        generate(
            "const modal = useTemplateRef(\n  'modal')\nuseTemplateRef<{\n  close: () => void\n}>('typed')\nif (ok) {\n  useTemplateRef('nested')\n}\n"
        ),
        "void (null! as __VizeTemplateRefs[\"modal\"]); const modal = __vizeInferRef(\n  'modal')\nuseTemplateRef<{\n  close: () => void\n}>('typed')\nvoid (null! as __VizeTemplateRefs[\"nested\"]); if (ok) {\n  __vizeInferRef('nested')\n}\n"
    );
}

#[test]
fn a_statement_that_shares_its_first_line_keeps_existing_key_check_ownership() {
    assert_eq!(
        generate(
            "const a = [\n  1]; const b = useTemplateRef('b')\n  const c = useTemplateRef('c'), d = useTemplateRef('d')\n"
        ),
        "const a = [\n  1]; const b = __vizeInferRef('b')\nvoid (null! as __VizeTemplateRefs[\"c\"]); void (null! as __VizeTemplateRefs[\"d\"]);   const c = __vizeInferRef('c'), d = __vizeInferRef('d')\n"
    );
}

#[test]
fn helper_names_avoid_authored_bindings_and_unresolved_reads() {
    for prefix in ["const __vizeInferRef = 1;", "void __vizeInferRef;"] {
        let script =
            format!("{prefix}\nvoid __vize00000000;\nconst box = useTemplateRef('box');\n");
        let usage = usage(&script);
        assert_eq!(usage.helper_name(), "__vize00000001");
        assert_eq!(usage.helper_name().len(), "useTemplateRef".len());
        assert_eq!(
            generate(&script),
            format!(
                "{prefix}\nvoid __vize00000000;\nvoid (null! as __VizeTemplateRefs[\"box\"]); const box = __vize00000001('box');\n"
            )
        );
    }
}

#[test]
fn authored_helpers_and_nested_shadow_bindings_are_not_specialized() {
    for script in [
        "function useTemplateRef(key: string) { return key; }\nconst ref = useTemplateRef('user');",
        "import { useTemplateRef } from 'other';\nconst ref = useTemplateRef('user');",
        "import { useTemplateRef } from 'vue';\n{ const useTemplateRef = (key: string) => key; useTemplateRef('user'); }",
    ] {
        let plan = SetupHelperPlan::collect(&vize_croquis::Croquis::default(), Some(script), false);
        assert!(!plan.template_refs.needs_registry(), "{script}");
    }
    let script = "import { useTemplateRef } from 'vue';\nconst ref = useTemplateRef('box');";
    let plan = SetupHelperPlan::collect(&vize_croquis::Croquis::default(), Some(script), false);
    assert!(plan.template_refs.needs_registry());
}

#[test]
fn vue_import_aliases_keep_exact_source_positions_after_short_callees() {
    let script = "import { useTemplateRef as r } from 'vue';\nconst button = r('button'); const typed = r<HTMLElement>('typed');\n";
    let usage = usage(script);
    assert!(usage.needs_registry());
    let start = script.find("const button").unwrap();
    let mut line = std::borrow::Cow::Borrowed(&script[start..]);
    let spans = usage
        .key_checks(true)
        .specialize_line(&mut line, start, 100, &|offset| offset + 20);
    assert_eq!(
        line,
        "const button = __vizeInferRef('button'); const typed = r<HTMLElement>('typed');\n"
    );
    for span in spans {
        let generated = &line[span.gen_range.start - 100..span.gen_range.end - 100];
        let authored = &script[span.src_range.start - 20..span.src_range.end - 20];
        assert!(generated == authored || generated == "__vizeInferRef" && authored == "r");
    }
}

#[test]
fn rewritten_aliases_keep_only_their_used_imports_live() {
    let script = "import { useTemplateRef as r, useTemplateRef as unused } from 'vue';\nconst first = r('first'); const second = r('second');\n";
    let plan = usage(script);
    let mut anchors = String::default();
    plan.emit_import_anchors(&mut anchors);
    assert_eq!(anchors, "void r;\n");
    let mut implicit = String::default();
    usage("const first = useTemplateRef('first');").emit_import_anchors(&mut implicit);
    assert!(implicit.is_empty());
}

#[test]
fn escaped_composable_callees_preserve_authored_spelling() {
    let script = "const target = useTemplateRe\\u0066('target');\n";
    assert_eq!(
        generate(script),
        "void (null! as __VizeTemplateRefs[\"target\"]); const target = __vizeInferRef('target');\n"
    );
}

#[test]
fn ambient_callees_remain_owned_and_only_real_vue_contracts_are_specialized() {
    let script = "const target = useTemplateRe\\u0066('target');\n";
    let usage = usage(script);
    let mut checks = usage.key_checks(true);
    let mut generated = String::default();
    let mut mappings = Vec::new();
    checks.emit_for_line(&mut generated, &mut mappings, (0, script.len()), &|n| n);
    checks.emit_line(
        &mut generated,
        &mut mappings,
        std::borrow::Cow::Borrowed(script),
        (0, script.len()),
        &|n| n,
    );
    assert_eq!(
        generated,
        "void (null! as (__vizeInferRefIsVue extends true ? __VizeTemplateRefs : Record<string, never>)[\"target\"]); const target = (useTemplateRe\\u0066 as __vizeInferRefGlobal)('target');\n\n"
    );
    let callee = mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| &script[span.src_range.clone()] == "useTemplateRe\\u0066")
        .unwrap();
    assert_eq!(&generated[callee.gen_range.clone()], "useTemplateRe\\u0066");
}

#[test]
fn generated_global_type_names_cannot_capture_authored_names() {
    for suffix in ["Source", "Equal", "Result", "IsVue", "Global"] {
        let script = format!("type __vizeInferRef{suffix} = string;\nuseTemplateRef('target');");
        assert_ne!(usage(&script).helper_name(), "__vizeInferRef");
    }
}
