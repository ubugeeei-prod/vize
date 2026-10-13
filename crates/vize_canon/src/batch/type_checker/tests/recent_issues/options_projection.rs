//! Genuine whole native counterfactuals for typed Options and denied setup-only JS.
use super::{real_vue::create_project_case, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeChecker, SfcBlockType, TypeChecker};

const CHILD: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/OptionsChild.vue.txt"
);
const WRONG: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/OptionsParent.vue.txt"
);
type WholeDiagnostic = (
    std::path::PathBuf,
    u32,
    u32,
    vize_carton::String,
    Option<u32>,
    u8,
    Option<SfcBlockType>,
);

fn native_vector(
    name: &str,
    child: Option<&str>,
    parent: &str,
    options: bool,
    legacy: bool,
    dialect: vize_carton::config::VueVersion,
    allowed: bool,
) -> Vec<WholeDiagnostic> {
    let mut authored = vec![("src/Parent.vue", parent)];
    if let Some(child) = child {
        authored.push(("src/Child.vue", child));
    }
    let project = create_project_case(name, &authored);
    let config = vize_carton::cstr!(
        "{{\"compilerOptions\":{{\"strict\":true,\"target\":\"ES2022\",\"module\":\"ESNext\",\"moduleResolution\":\"bundler\",\"skipLibCheck\":true,\"allowJs\":{allowed},\"checkJs\":false}},\"include\":[\"src/**/*\"]}}"
    );
    std::fs::write(project.join("tsconfig.json"), config.as_str()).unwrap();
    let vue = std::fs::read_to_string(project.join("node_modules/vue/package.json")).unwrap();
    let mut checker = BatchTypeChecker::new(&project).unwrap();
    if options {
        checker.enable_options_api();
    }
    if legacy {
        checker.enable_legacy_vue2();
    }
    checker.set_dialect(dialect);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    eprintln!(
        "Options JS whole native case={name} config={config} vue={vue} original_diagnostics={:#?}",
        result.diagnostics
    );
    let whole = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            // Only the project-root path is normalized; all seven fields remain.
            let file = diagnostic
                .file
                .strip_prefix(&project)
                .unwrap()
                .to_path_buf();
            (
                file,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message,
                diagnostic.code,
                diagnostic.severity,
                diagnostic.block_type,
            )
        })
        .collect::<Vec<_>>();
    std::fs::remove_dir_all(project).unwrap();
    whole
}

#[test]
fn typed_options_plain_and_mixed_props_reject_wrong_and_accept_repair_while_setup_js_stays_denied()
{
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let mixed = CHILD.replace(
        "<template>",
        "<script setup>const local = 1;</script>\n<template>",
    );
    let repaired = WRONG.replace("'wrong'", "1");
    for (mode, options, legacy, dialect) in [
        ("options", true, false, vize_carton::config::VueVersion::V3),
        ("legacy", false, true, vize_carton::config::VueVersion::V3),
        ("vue2", false, false, vize_carton::config::VueVersion::V2),
        ("vue27", false, false, vize_carton::config::VueVersion::V2_7),
    ] {
        for (kind, child) in [
            ("plain", CHILD),
            ("mixed", mixed.as_str()),
            (
                "setup-only",
                "<script setup>defineProps({ count: Number });</script><template><span>{{ count }}</span></template>",
            ),
        ] {
            for (phase, parent) in [("wrong", WRONG), ("repaired", repaired.as_str())] {
                let name = vize_carton::cstr!("options-javascript-{mode}-{kind}-{phase}");
                let reference_name = vize_carton::cstr!("{name}-allowed-reference");
                let reference_child = if kind == "setup-only" {
                    None
                } else {
                    Some(child)
                };
                let expected = native_vector(
                    reference_name.as_str(),
                    reference_child,
                    parent,
                    options,
                    legacy,
                    dialect,
                    true,
                );
                let actual = native_vector(
                    name.as_str(),
                    Some(child),
                    parent,
                    options,
                    legacy,
                    dialect,
                    false,
                );
                eprintln!(
                    "Options JS provider native={} case={name} whole_reference={expected:#?} whole_actual={actual:#?}",
                    native.display()
                );
                if kind == "setup-only" || phase == "wrong" {
                    let code = if kind == "setup-only" { 2307 } else { 2322 };
                    let block = if kind == "setup-only" {
                        SfcBlockType::ScriptSetup
                    } else {
                        SfcBlockType::Template
                    };
                    assert_eq!(
                        expected
                            .iter()
                            .map(|value| (value.0.clone(), value.4, value.5, value.6))
                            .collect::<Vec<_>>(),
                        vec![(
                            std::path::PathBuf::from("src/Parent.vue"),
                            Some(code),
                            1,
                            Some(block)
                        )]
                    );
                } else {
                    assert_eq!(expected, Vec::<WholeDiagnostic>::new());
                }
                assert_eq!(
                    actual, expected,
                    "complete native vector for {mode}/{kind}/{phase}"
                );
            }
        }
    }
}
