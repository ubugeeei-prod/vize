use super::NoDupeKeysModuleScript;
use crate::rules::script::{
    NoDupeKeys, ScriptLintResult, ScriptRule, SfcScriptContext, script_source_type,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;

fn collisions(setup: &str, module: Option<&str>) -> Vec<vize_l0::CompactString> {
    let setup_alloc = Allocator::default();
    let module_alloc = Allocator::default();
    let parsed = Parser::new(&setup_alloc, setup, script_source_type()).parse();
    assert!(parsed.diagnostics.is_empty());
    let sibling =
        module.map(|source| Parser::new(&module_alloc, source, script_source_type()).parse());
    let module = sibling
        .as_ref()
        .map_or(NoDupeKeysModuleScript::Absent, |parsed| {
            assert!(parsed.diagnostics.is_empty());
            NoDupeKeysModuleScript::Parsed {
                program: &parsed.program,
                offset: 0,
            }
        });
    let mut result = ScriptLintResult::default();
    NoDupeKeys.check_setup_with_module(&parsed.program, setup, 0, module, &mut result);
    result
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect()
}

#[test]
fn local_inheritance_and_declaration_merging_keep_one_prop_namespace() {
    for (setup, module) in [
        (
            "interface Base { foo: string }; interface Props extends Base {}; defineProps<Props>(); const foo = 1;",
            None,
        ),
        (
            "defineProps<Props>(); const foo = 1;",
            Some("export interface Base { foo: string }; export interface Props extends Base {}"),
        ),
        (
            "interface Base { foo: string }; interface Props extends Base { foo: string }; defineProps<Props>(); const foo = 1;",
            None,
        ),
        (
            "interface Props { foo: string }; interface Props { bar: string }; defineProps<Props>(); const foo = 1, bar = 2;",
            None,
        ),
    ] {
        let actual = collisions(setup, module);
        assert_eq!(
            actual
                .iter()
                .filter(|message| message.as_str() == "Duplicated key 'foo'")
                .count(),
            1
        );
        if setup.contains("const foo = 1, bar = 2") {
            assert_eq!(actual.len(), 2);
        } else {
            assert_eq!(actual.len(), 1);
        }
    }
}

#[test]
fn closer_type_and_runtime_owners_prevent_sibling_fallback() {
    for setup in [
        "class Props {}; defineProps<Props>(); const foo = 1;",
        "function declarations() {}; defineProps(declarations); const foo = 1;",
        "class declarations {}; defineProps(declarations); const foo = 1;",
        "if (true) { var declarations = external }; defineProps(declarations); const foo = 1;",
    ] {
        assert!(collisions(setup, Some("export interface Props { foo: string }; export const declarations = {foo: String};")).is_empty(), "{setup}");
    }
}

#[test]
fn setup_extension_keeps_every_legacy_options_diagnostic_field() {
    let source = "export default { props: ['foo'], data() { return {foo: 1} } }";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, script_source_type()).parse();
    let mut before = ScriptLintResult::default();
    let mut after = ScriptLintResult::default();
    NoDupeKeys.check_program(&parsed.program, source, 37, &mut before);
    NoDupeKeys.check_program_with_sfc(
        &parsed.program,
        source,
        37,
        SfcScriptContext {
            is_sfc: true,
            is_script_setup: true,
            sole_script_block: true,
            ..Default::default()
        },
        &mut after,
    );
    assert_eq!(before.error_count, 1);
    assert_eq!(format!("{before:?}"), format!("{after:?}"));
}
