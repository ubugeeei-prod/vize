use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_define_props_typeof_setup_binding_deferred_to_setup_scope() {
    let script = r#"const someDefinition = {
  foo: 'fooVal',
} as const;

type SomeGenericType<T extends Record<string, unknown>> = {
  baz: string;
  items: T;
};

const props = defineProps<SomeGenericType<typeof someDefinition>>();
"#;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default());
    let (module_scope, setup_and_after) = output
        .code
        .split_once("// ========== Setup Scope ==========")
        .expect("setup scope marker present");

    assert!(
        !module_scope.contains("typeof someDefinition"),
        "setup-scope value must not be referenced from module scope:\n{}",
        output.code
    );
    assert!(
        setup_and_after.contains("type __VizeSetupProps = SomeGenericType<typeof someDefinition>;"),
        "setup-local props artifact should preserve the concrete props type:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "export type Props = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];"
        ),
        "module Props should be exported through __setup return type:\n{}",
        output.code
    );
}

#[test]
fn test_deferred_define_props_does_not_redeclare_hoisted_props_type() {
    let script = r#"interface Props {
  baz: string;
}

const someDefinition = {
  foo: 'fooVal',
} as const;

defineProps<Props & { items: typeof someDefinition }>();
"#;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default());

    assert!(
        output.code.contains("interface Props {\n  baz: string;\n}"),
        "existing hoisted Props should remain in module scope:\n{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("export type Props = Awaited<ReturnType<typeof __setup>>"),
        "deferred props must not redeclare an existing module-scope Props:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "type __VizeResolvedProps = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];"
        ),
        "default export should use a private resolved props alias:\n{}",
        output.code
    );
    assert!(
        output.code.contains("  $props: __VizeResolvedProps;"),
        "component instance should use the full deferred props type:\n{}",
        output.code
    );
}

#[test]
fn test_define_props_non_hoisted_type_ref_deferred_to_setup_scope() {
    let script = r#"type WidgetProps = {
  items: typeof widgetDefinitions;
};

const widgetDefinitions = {
  clock: {},
} as const;

defineProps<WidgetProps>();
"#;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default());
    let (module_scope, setup_and_after) = output
        .code
        .split_once("// ========== Setup Scope ==========")
        .expect("setup scope marker present");

    assert!(
        !module_scope.contains("WidgetProps"),
        "non-hoisted setup type must not be referenced from module scope:\n{}",
        output.code
    );
    assert!(
        setup_and_after.contains("type __VizeSetupProps = WidgetProps;"),
        "setup-local props artifact should reference the non-hoisted type:\n{}",
        output.code
    );
}
