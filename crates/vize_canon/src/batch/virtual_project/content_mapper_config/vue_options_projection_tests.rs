//! Every check option reaches the Content Mapper projection (#11688).
//!
//! The mapper read only part of `vueCompilerOptions`, so the hosts that check
//! through it (tsc, tsrs, the editor's backend A) silently missed errors that
//! `vize check` reported. Each row loads a real tsconfig, generates the
//! projection with and without one option, and requires them to differ.

use std::path::Path;

use super::super::generate_vue_content_mapper_transform_with_options;
use super::super::vue_check_options::CHECK_OPTION_KEYS;
use super::ContentMapperProjectConfiguration;

const TEMPLATE_GLOBALS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vue-compiler-options-mapper-11688/TemplateGlobals.vue.txt"
));

const VAPOR_DIRECTIVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vue-compiler-options-mapper-11688/VaporDirective.vue.txt"
));

const COMPONENT_ROOT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vue-compiler-options-mapper-11688/ComponentRoot.vue.txt"
));

const STYLE_CLASSES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vue-compiler-options-mapper-11688/StyleClasses.vue.txt"
));

const STYLE_IMPORT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/vue-compiler-options-mapper-11688/StyleImport.vue.txt"
));

/// (option, options both projections share, probe).
const ROWS: &[(&str, &str, &str)] = &[
    ("strictTemplates", "", TEMPLATE_GLOBALS),
    ("checkUnknownProps", "", TEMPLATE_GLOBALS),
    ("checkUnknownComponents", "", TEMPLATE_GLOBALS),
    ("checkUnknownEvents", "", TEMPLATE_GLOBALS),
    ("checkUnknownDirectives", "", TEMPLATE_GLOBALS),
    ("strictVModel", "", TEMPLATE_GLOBALS),
    ("strictCssModules", "", TEMPLATE_GLOBALS),
    (
        "strictComponentAttrs",
        r#""checkUnknownProps": true"#,
        COMPONENT_ROOT,
    ),
    ("vapor", "", VAPOR_DIRECTIVE),
    ("jsxSlots", "", TEMPLATE_GLOBALS),
    ("inferComponentDollarEl", "", TEMPLATE_GLOBALS),
    ("inferTemplateDollarEl", "", TEMPLATE_GLOBALS),
    ("inferComponentDollarRefs", "", TEMPLATE_GLOBALS),
    ("inferTemplateDollarRefs", "", TEMPLATE_GLOBALS),
    ("inferTemplateDollarSlots", "", TEMPLATE_GLOBALS),
    ("inferTemplateDollarAttrs", "", TEMPLATE_GLOBALS),
    ("fallthroughAttributes", "", COMPONENT_ROOT),
    (
        "checkRequiredFallthroughAttributes",
        r#""fallthroughAttributes": true"#,
        COMPONENT_ROOT,
    ),
    ("resolveStyleClassNames", "", STYLE_CLASSES),
    ("resolveStyleImports", "", STYLE_IMPORT),
];

fn projection(vue_compiler_options: &str, probe: &str) -> vize_l0::String {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(
        &path,
        vize_l0::cstr!(r#"{{"vueCompilerOptions":{{{vue_compiler_options}}}}}"#).as_str(),
    )
    .unwrap();
    let configuration = ContentMapperProjectConfiguration::load(&path).unwrap();
    generate_vue_content_mapper_transform_with_options(
        Path::new("src/App.vue"),
        probe,
        configuration.transform_options,
    )
    .unwrap()
    .text
}

#[test]
fn every_check_option_changes_the_mapper_projection() {
    let mut ignored = Vec::new();
    for &(key, shared, probe) in ROWS {
        let separator = if shared.is_empty() { "" } else { ", " };
        let with = vize_l0::cstr!(r#"{shared}{separator}"{key}": true"#);
        if projection(&with, probe) == projection(shared, probe) {
            ignored.push(key);
        }
    }
    assert!(ignored.is_empty(), "the mapper ignores {ignored:?}");
}

#[test]
fn every_check_option_has_a_projection_row() {
    let rows: Vec<_> = ROWS.iter().map(|(key, _, _)| *key).collect();
    assert_eq!(rows, CHECK_OPTION_KEYS);
}
