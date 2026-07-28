use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::{
    VirtualTsOptions, generate_virtual_ts, generate_virtual_ts_with_offsets_legacy_vue2,
};

#[test]
fn native_prop_check_uses_vue_jsx_type_and_authored_subspans() {
    let script = r#"const disabledFlag: string = "yes""#;
    let template = r#"<button type="button" :disabled="disabledFlag">go</button>"#;
    let allocator = vize_carton::Bump::new();
    let (root, summary) = analyze(&allocator, script, template, false);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("__VizeNativeElement<import('vue').NativeElements, \"button\">"),
        "native check should derive the element from Vue's native element contract:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "__VizeNativeElementProp<__VizeNativeElement<import('vue').NativeElements, \"button\">, \"disabled\"> = (disabledFlag);"
        ),
        "bound disabled should be assigned to the exact Vue prop type:\n{}",
        output.code
    );

    let name_start = template.find("disabled").expect("prop name");
    let name_range = name_start..name_start + "disabled".len();
    let name_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == name_range)
        .expect("native prop check should anchor at the authored prop name");
    assert!(
        output.code[name_span.gen_range.clone()].starts_with("__vize_native_prop_check_"),
        "synthetic check identifier should carry the prop-type diagnostic"
    );

    let value_start = template.find("disabledFlag").expect("bound expression");
    let value_range = value_start..value_start + "disabledFlag".len();
    let value_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == value_range)
        .expect("native prop initializer should retain its authored expression range");
    assert_eq!(&output.code[value_span.gen_range.clone()], "disabledFlag");
}

#[test]
fn native_prop_check_ignores_non_boolean_names_dynamic_names_and_components() {
    let script = "import Child from './Child.vue'\nconst name = 'disabled'\nconst value = 'yes'";
    let template = r#"<button :title="value" :[name]="value" /><Child :disabled="value" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, summary) = analyze(&allocator, script, template, false);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__vize_native_prop_check_"),
        "non-boolean and dynamic native attrs plus child props must keep their existing paths:\n{}",
        output.code
    );
    assert!(
        output.code.contains("__vize_prop_check_"),
        "the child component should still use component prop checking:\n{}",
        output.code
    );
}

#[test]
fn legacy_vue2_does_not_require_vue_native_elements() {
    let script = r#"const disabledFlag: string = "yes""#;
    let template = r#"<button :disabled="disabledFlag">go</button>"#;
    let allocator = vize_carton::Bump::new();
    let (root, summary) = analyze(&allocator, script, template, true);
    let output = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    assert!(
        !output.code.contains("import('vue').NativeElements"),
        "Vue 2 virtual TS must not require the Vue 3 native element contract:\n{}",
        output.code
    );
}

fn analyze<'a>(
    allocator: &'a vize_carton::Bump,
    script: &str,
    template: &'a str,
    legacy_vue2: bool,
) -> (vize_relief::RootNode<'a>, vize_croquis::Croquis) {
    let (root, _) = vize_armature::parse(allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if legacy_vue2 {
        analyzer = analyzer.with_legacy_vue2();
    }
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    (root, analyzer.finish())
}
