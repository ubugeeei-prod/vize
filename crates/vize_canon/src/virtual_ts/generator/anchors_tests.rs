use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::String;
use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/quoted-prop-setup-anchors/"
);

fn parse_errors(source: &str) -> usize {
    Parser::new(&Allocator::default(), source, SourceType::ts())
        .parse()
        .errors
        .len()
}

fn project(source: &str) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("authored full SFC");
    let script = descriptor
        .script_setup
        .as_ref()
        .expect("script setup")
        .content
        .as_ref();
    assert_eq!(parse_errors(script), 0, "authored TypeScript must parse");
    let template = descriptor
        .template
        .as_ref()
        .expect("template")
        .content
        .as_ref();
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(
        errors.is_empty(),
        "authored template must parse: {errors:?}"
    );
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts(&analyzer.finish(), Some(script), Some(&root), 0).code
}

#[test]
fn original_element_plus_quoted_props_keep_virtual_typescript_parseable() {
    for name in ["popover", "tooltip"] {
        let source = std::fs::read_to_string(format!("{CORPUS}{name}.vue.txt")).unwrap();
        let before =
            std::fs::read_to_string(format!("{CORPUS}{name}.original-h.vue.ts.txt")).unwrap();
        assert!(
            parse_errors(&before) > 0,
            "literal original-H projection is the regression control"
        );
        assert!(before.contains("void onUpdate:visible;"));
        let after = project(&source);
        assert_eq!(parse_errors(&after), 0, "full {name} projection must parse");
        assert!(!after.contains("void onUpdate:visible;"));
        if name == "popover" {
            assert!(
                after.contains("void updateEventKeyRaw;"),
                "real setup binding must remain anchored"
            );
            assert!(
                after.contains("onUpdate:visible"),
                "authored prop and literal must remain present"
            );
        }
    }
}

#[test]
fn quoted_prop_keys_keep_valid_unicode_and_dollar_identifiers() {
    let source = r#"<script setup lang="ts">
const props = defineProps<{ "onUpdate:visible": () => void; "data-id": string; "class": string; "default": string; café: string; $value: string; _private: string }>()
const résumé = props.café
const $setup = props.$value
</script>
<template><div>{{ props["onUpdate:visible"] }}{{ props["data-id"] }}{{ props.class }}{{ props.default }}{{ props.café }}{{ props.$value }}{{ props._private }}{{ résumé }}{{ $setup }}</div></template>"#;
    let output = project(source);
    assert_eq!(parse_errors(&output), 0);
    for name in ["café", "$value", "_private", "résumé", "$setup"] {
        assert!(
            output.contains(&format!("void {name};")),
            "valid prop identifier {name}"
        );
    }
    assert!(!output.contains("void data-id;"));
    assert!(!output.contains("void onUpdate:visible;"));
    assert!(!output.contains("void class;"));
    assert!(!output.contains("void default;"));
    for name in ["onUpdate:visible", "data-id", "class", "default"] {
        assert!(
            output.contains(name),
            "authored property {name} must remain"
        );
    }
}
