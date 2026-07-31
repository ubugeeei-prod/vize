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
            .contains("type __VizeNativeElements = import('vue').NativeElements;"),
        "native check should derive the element from Vue's native element contract:\n{}",
        output.code
    );
    // The module is named exactly once. A Vue without `NativeElements` then
    // makes that single alias an error type, which propagates as `any` and
    // leaves the checks permissive, instead of reporting `TS2694` per binding.
    assert_eq!(
        output.code.matches("import('vue').NativeElements").count(),
        1
    );
    assert!(
        output.code.contains(
            "__VizeNativeElementProp<__VizeNativeElement<\"button\">, \"disabled\"> = (disabledFlag);"
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

/// Every statically named `v-bind` on a native element is checked against that
/// element's own prop type, which is what `vue-tsc` does: it reports `TS2322`
/// for `<a :href="1">` just as it does for `<button :disabled="'yes'">`.
/// Restricting the check to boolean-ish attribute names silently dropped every
/// other native prop-type error.
#[test]
fn native_prop_check_covers_every_static_attribute_name() {
    let script = "const value = 'yes'";
    let template = r#"<a :href="value" :tabindex="value" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, summary) = analyze(&allocator, script, template, false);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    for name in ["href", "tabindex"] {
        let expected = vize_carton::cstr!(
            "__VizeNativeElementProp<__VizeNativeElement<\"a\">, \"{name}\"> = (value);"
        );
        assert!(
            output.code.contains(expected.as_str()),
            "`:{name}` must be checked against the anchor element's own prop type:\n{}",
            output.code
        );
    }
}

/// A dynamic attribute name has no statically known prop to check against, and
/// a child component keeps its own generic prop-checker path.
#[test]
fn native_prop_check_ignores_dynamic_names_and_components() {
    let script = "import Child from './Child.vue'\nconst name = 'disabled'\nconst value = 'yes'";
    let template = r#"<button :[name]="value" /><Child :disabled="value" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, summary) = analyze(&allocator, script, template, false);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__vize_native_prop_check_"),
        "dynamic native attrs plus child props must keep their existing paths:\n{}",
        output.code
    );
    assert!(
        output.code.contains("__vize_prop_check_"),
        "the child component should still use component prop checking:\n{}",
        output.code
    );
}

/// The native element table is declared once, guarded against a `vue` that has
/// no `NativeElements`, and kept referenced so it cannot be reported as unused —
/// in the per-file preamble and in the hoisted ambient helpers alike.
///
/// Both failure modes were live defects:
///
/// - `TS2694`/`TS2307` when the installed `vue` has no `NativeElements`. The
///   checks already degraded to `any`, but the alias still reported against the
///   helpers file. `vize check` drops that (the helpers text maps to no authored
///   source) while `vize check --declaration` treats Corsa's non-zero exit as
///   fatal, so it aborted the whole emit. `// @ts-ignore` covers this one.
/// - `TS6196` "declared but never used" on `__VizeNativeElement` and
///   `__VizeNativeElementProp` for a program that binds no native prop, which
///   reached `check-server` clients as unmapped hints. `@ts-ignore` does *not*
///   cover this one — it filters errors, not the suggestion channel — so the
///   ambient `__vizeNativeElementProp` declaration keeps both aliases referenced.
#[test]
fn native_element_table_is_guarded_and_kept_referenced() {
    const GUARD: &str = "// @ts-ignore TS2694/TS2307: a `vue` without `NativeElements` must degrade native prop checks to unchecked, never error. See virtual_ts/expressions/native_props.rs.";
    const ALIAS: &str = "type __VizeNativeElements = import('vue').NativeElements;";
    const REFERENCE: &str = "declare function __vizeNativeElementProp<__Tag extends PropertyKey, __Prop extends PropertyKey>(value: __VizeNativeElementProp<__VizeNativeElement<__Tag>, __Prop>): void;";

    for (name, text) in [
        (
            "SHARED_PREAMBLE_DTS",
            crate::virtual_ts::SHARED_PREAMBLE_DTS,
        ),
        (
            "VUE_TYPE_HELPERS",
            crate::virtual_ts::helpers::VUE_TYPE_HELPERS,
        ),
    ] {
        let lines = text.lines().collect::<Vec<_>>();
        let alias_lines = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| **line == ALIAS)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        assert_eq!(
            alias_lines.len(),
            1,
            "{name} must name Vue's native element table exactly once"
        );
        assert_eq!(lines[alias_lines[0] - 1], GUARD, "{name} must guard it");
        assert_eq!(
            lines.iter().filter(|line| **line == REFERENCE).count(),
            1,
            "{name} must keep the conditional aliases referenced exactly once"
        );
    }
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
