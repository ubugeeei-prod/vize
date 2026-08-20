use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

mod define_model_modifiers;
mod dynamic_component_props;

#[test]
fn generic_props_call_merges_static_and_dynamic_class_bindings() {
    let script = r#"import TeacherCard from "./TeacherCard.vue"
const teacher = { name: "Ada" }
const isLoading = false
"#;
    let template = r#"<TeacherCard
  class="ma-1"
  :teacher="teacher"
  :class="{ 'loading-place-holder': isLoading }"
/>"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("\"class\": [\"ma-1\", { 'loading-place-holder': isLoading }]"),
        "expected merged class binding in generic props call:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches("\"class\":").count(),
        1,
        "class should be emitted once in the props object:\n{}",
        output.code
    );
}

#[test]
fn component_prop_check_anchors_name_and_preserves_bound_expression() {
    let script = r#"import Child from "./Child.vue"
const benchmarkMirror = 1
"#;
    let template = r#"<Child :code="String(benchmarkMirror)" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    // The synthetic identifier — where the child prop-type error lands —
    // anchors at the attribute name, matching vue-tsc.
    let name_start = template.find(":code").expect("attribute present") + 1;
    let name_range = name_start..name_start + "code".len();
    let name_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == name_range)
        .expect("component prop check should anchor at the attribute name");
    assert!(
        output.code[name_span.gen_range.clone()].starts_with("__vize_prop_check_"),
        "synthetic check identifier should map to the attribute name: {name_span:?}"
    );

    // The initializer keeps the exact authored expression range so errors
    // inside the value land on the authored bytes.
    let expression = "String(benchmarkMirror)";
    let source_start = template.find(expression).expect("bound expression present");
    let source_range = source_start..source_start + expression.len();
    let value_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == source_range)
        .expect("component prop check should preserve the exact bound expression range");
    assert_eq!(
        &output.code[value_span.gen_range.clone()],
        expression,
        "initializer sub-span should cover the generated expression"
    );
}

#[test]
fn static_attribute_values_are_type_checked_like_dynamic_bindings() {
    let script = r#"import HelloWorld from "./HelloWorld.vue"
"#;
    let template = r#"<HelloWorld msg="You did it!" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains("type __HelloWorld_0_prop_msg = __VizePropValue<__HelloWorld_ValueProps_0, 'msg', __HelloWorld_FallthroughValue_0<'msg'>>;"),
        "static prop must declare its child prop type alias:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(": __HelloWorld_0_prop_msg = \"You did it!\";"),
        "static prop value must be asserted against the child prop type:\n{}",
        output.code
    );

    // The synthetic check identifier anchors at the attribute name, and the
    // initializer keeps the authored value range.
    let name_start = template.find("msg=").expect("attribute present");
    let name_range = name_start..name_start + "msg".len();
    let name_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == name_range)
        .expect("static prop check should anchor at the attribute name");
    assert!(
        output.code[name_span.gen_range.clone()].starts_with("__vize_prop_check_"),
        "synthetic check identifier should map to the attribute name: {name_span:?}"
    );

    let value_start = template.find("You did it!").expect("static value present");
    let value_range = value_start..value_start + "You did it!".len();
    assert!(
        output
            .mappings
            .iter()
            .flat_map(|mapping| &mapping.sub_spans)
            .any(|span| span.src_range == value_range),
        "static prop check should keep the authored value range"
    );
}

#[test]
fn static_attribute_values_escape_into_exact_string_literals() {
    let script = r#"import Child from "./Child.vue"
"#;
    let template = "<Child label='say \"hi\" \\ done' />";

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        output
            .code
            .contains(": __Child_0_prop_label = \"say \\\"hi\\\" \\\\ done\";"),
        "static value must escape quotes and backslashes:\n{}",
        output.code
    );
}

#[test]
fn valueless_static_attributes_stay_out_of_per_prop_checks() {
    let script = r#"import Child from "./Child.vue"
"#;
    let template = r#"<Child disabled />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        !output.code.contains("__Child_0_prop_disabled ="),
        "valueless attributes keep their boolean-shorthand semantics:\n{}",
        output.code
    );
}

#[test]
fn repeated_prop_names_keep_unique_checks_and_one_type_alias() {
    let script = r#"import Child from "./Child.vue"
const isLoading = false
"#;
    let template = r#"<Child class="static-card" :class="{ loading: isLoading }" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert_eq!(
        output
            .code
            .matches("type __Child_0_prop_class = __VizePropValue<__Child_ValueProps_0, 'class', __Child_FallthroughValue_0<'class'>>;")
            .count(),
        1,
        "the child prop type alias must be declared exactly once:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const __vize_prop_check_0_class: __Child_0_prop_class = \"static-card\";"),
        "the static class value keeps the base check name:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "const __vize_prop_check_0_class_2: __Child_0_prop_class = { loading: isLoading };"
        ),
        "the bound class value gets a unique check name:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const __vize_prop_check_0_class")
            .count(),
        2,
        "both authored values stay checked:\n{}",
        output.code
    );
}

#[test]
fn attribute_name_anchor_survives_prefixes_and_modifiers() {
    let script = r#"import Child from "./Child.vue"
const bind = 1
const sync = 2
const fooBar = 3
"#;
    let template = r#"<Child v-bind:bind="bind" :sync.camel="sync" :foo-bar="fooBar" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let sub_spans: Vec<_> = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .collect();

    let bind_name = template.find("v-bind:bind").expect("v-bind attr") + "v-bind:".len();
    assert!(
        sub_spans.iter().any(
            |span| span.src_range == (bind_name..bind_name + "bind".len())
                && output.code[span.gen_range.clone()].starts_with("__vize_prop_check_")
        ),
        "v-bind: prefixed name should anchor after the prefix"
    );

    let sync_name = template.find(":sync.camel").expect("modifier attr") + 1;
    assert!(
        sub_spans.iter().any(
            |span| span.src_range == (sync_name..sync_name + "sync".len())
                && output.code[span.gen_range.clone()].starts_with("__vize_prop_check_")
        ),
        "modifier attributes should anchor at the name, not the modifier"
    );

    let kebab_name = template.find(":foo-bar").expect("kebab attr") + 1;
    assert!(
        sub_spans.iter().any(
            |span| span.src_range == (kebab_name..kebab_name + "foo-bar".len())
                && output.code[span.gen_range.clone()].starts_with("__vize_prop_check_")
        ),
        "kebab-case names should anchor across the whole hyphenated name, not a leading segment"
    );
}

#[test]
fn attribute_name_anchor_uses_utf8_byte_offsets_after_multibyte_text() {
    let script = r#"import Child from "./Child.vue"
const total = 1
"#;
    // The static label holds multibyte characters (😀 is 4 UTF-8 bytes / 2 UTF-16
    // units, ハ is 3 UTF-8 bytes) so a byte/char confusion in the source mapping
    // would shift every following range.
    let template = r#"<Child label="😀ハ" :count="total" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let sub_spans: Vec<_> = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .collect();

    // `str::find` returns UTF-8 byte offsets; the multibyte prefix makes the byte
    // offset strictly larger than the char count, so these ranges only match when
    // the source mapping is byte-indexed.
    let name_start = template.find(":count").expect("attribute present") + 1;
    assert!(
        name_start > template[..name_start].chars().count(),
        "fixture must place the prop after multibyte text so byte and char offsets diverge"
    );
    let name_range = name_start..name_start + "count".len();
    let name_span = sub_spans
        .iter()
        .find(|span| span.src_range == name_range)
        .expect("prop check should anchor at the attribute name after multibyte text");
    assert!(
        output.code[name_span.gen_range.clone()].starts_with("__vize_prop_check_"),
        "synthetic check identifier should map to the attribute name: {name_span:?}"
    );

    // The initializer keeps the exact authored expression bytes.
    let value_start = template.find("total").expect("bound value present");
    let value_range = value_start..value_start + "total".len();
    assert!(
        sub_spans.iter().any(|span| span.src_range == value_range),
        "initializer sub-span should keep the authored value bytes after multibyte text"
    );
}
