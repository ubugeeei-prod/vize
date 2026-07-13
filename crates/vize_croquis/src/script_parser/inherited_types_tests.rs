use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{ScriptParseResult, analyze_script_setup_program_with_inherited_types, parse_script};

fn analyze_with_normal_script_types(normal: &str, setup: &str) -> ScriptParseResult {
    let inherited_types = parse_script(normal).types;
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        setup,
        SourceType::from_path("script.ts").expect("TypeScript source type should be valid"),
    )
    .parse();
    assert!(!parsed.panicked, "setup fixture should parse");

    analyze_script_setup_program_with_inherited_types(&parsed.program, setup, None, inherited_types)
}

fn prop_names(result: &ScriptParseResult) -> Vec<&str> {
    result
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect()
}

#[test]
fn setup_forward_interface_overrides_inherited_interface_for_define_props() {
    let result = analyze_with_normal_script_types(
        "interface Props { fromNormal: string }",
        r#"
const props = defineProps<Props>()

interface Props {
    fromSetup: number
}
"#,
    );

    assert_eq!(prop_names(&result), ["fromSetup"]);
}

#[test]
fn setup_forward_type_alias_overrides_inherited_alias_for_define_props() {
    let result = analyze_with_normal_script_types(
        "type Props = { fromNormal: string }",
        r#"
const props = defineProps<Props>()

type Props = {
    fromSetup: number
}
"#,
    );

    assert_eq!(prop_names(&result), ["fromSetup"]);
}
