use super::{exact_line_count, generate, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn assert_inferred_type_follows_pattern(authored: &str, pattern: &str, default: Option<&str>) {
    let template = format!(r#"<Slotty v-slot="{authored}">{{{{ 1 }}}}</Slotty>"#);
    let code = generate(template.as_str());
    let default = default.map_or_else(String::new, |value| format!(" = {value}"));
    let expected = format!(
        concat!(
            "  void function _slot_default_1({}: $SlotPayload<",
            "typeof __vize_slot_host_1, \"default\", ",
            "$RegisteredSlotProvider<import(\"vue\").GlobalComponents, ",
            "\"Slotty\", \"Slotty\", \"Slotty\">>{}) {{",
        ),
        pattern, default
    );
    assert_eq!(exact_line_count(&code, expected.as_str()), 1, "{code}");
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "the generated module must parse: {:?}\n{code}",
        parsed.diagnostics
    );
}

/// A whole-parameter default belongs after the inferred payload annotation.
/// Appending the annotation to the authored expression produced
/// `{ editor } = {}: $SlotPayload<...>`, which stops TypeScript before its
/// semantic pass and hides every real project diagnostic.
#[test]
fn defaulted_v_slot_pattern_keeps_the_slot_function_parseable() {
    let code = generate(r#"<Slotty v-slot="{ editor } = {}">{{ editor }}</Slotty>"#);
    let expected = concat!(
        "  void function _slot_default_1({ editor }: $SlotPayload<",
        "typeof __vize_slot_host_1, \"default\", ",
        "$RegisteredSlotProvider<import(\"vue\").GlobalComponents, ",
        "\"Slotty\", \"Slotty\", \"Slotty\">> = {}) {",
    );
    assert_eq!(
        exact_line_count(&code, expected),
        1,
        "the inferred payload type must precede the parameter default:\n{code}",
    );
    assert_eq!(
        code.matches("{ editor } = {}:").count(),
        0,
        "the invalid authored-default/type order must not survive:\n{code}",
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "the generated module must parse: {:?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn property_default_syntax_stays_inside_the_binding_pattern() {
    for (authored, pattern, default) in [
        (
            "{ make = () => 1, editor = fallback }",
            "{ make = () => 1, editor = fallback }",
            None,
        ),
        (
            r"{ matcher = /\}/, editor = fallback }",
            r"{ matcher = /\}/, editor = fallback }",
            None,
        ),
        (
            "{ flag = count > 0, editor = fallback }",
            "{ flag = count > 0, editor = fallback }",
            None,
        ),
        (
            "{ flag = count < 1 } = {}",
            "{ flag = count < 1 }",
            Some("{}"),
        ),
    ] {
        assert_inferred_type_follows_pattern(authored, pattern, default);
    }
}

#[test]
fn defaulted_v_slot_initializer_stays_mapped_to_the_authored_expression() {
    let template = r#"<Slotty v-slot="{ editor } = missingFallback">{{ editor }}</Slotty>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        None,
        Some(&root),
        0,
        0,
        &Default::default(),
    );
    let source_start = template.find("missingFallback").unwrap();
    let source_range = source_start..source_start + "missingFallback".len();
    let mapping = output
        .mapping
        .spans()
        .iter()
        .find(|mapping| mapping.src_range == source_range)
        .expect("the authored default initializer must retain a diagnostic mapping");
    assert_eq!(
        output.code.get(mapping.gen_range.clone()),
        Some("missingFallback")
    );
}

#[test]
fn default_mapping_uses_the_emitted_initializer_after_the_inferred_type() {
    let template = r#"<Slotty v-slot="{ editor } = Slotty">{{ editor }}</Slotty>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        None,
        Some(&root),
        0,
        0,
        &Default::default(),
    );
    let source_start = template.find("= Slotty").expect("authored initializer") + 2;
    let source_range = source_start..source_start + "Slotty".len();
    let mapping = output
        .mapping
        .spans()
        .iter()
        .find(|mapping| mapping.src_range == source_range)
        .expect("the authored initializer must retain a diagnostic mapping");
    assert_eq!(
        output
            .code
            .get(mapping.gen_range.start - 2..mapping.gen_range.end),
        Some("= Slotty")
    );
}
