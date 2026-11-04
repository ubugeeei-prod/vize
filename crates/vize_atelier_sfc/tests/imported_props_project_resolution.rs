use std::path::Path;

use oxc_ast::ast::{Expression, ObjectProperty, ObjectPropertyKind};
use oxc_ast_visit::{Visit, walk};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::{String, ToCompactString, cstr};

#[path = "imported_props_project_resolution/checked_imports.rs"]
mod checked_imports;

#[derive(Debug, PartialEq, Eq)]
struct RuntimeProp {
    name: String,
    constructor: Option<String>,
    required: Option<bool>,
}

#[derive(Default)]
struct RuntimeProps(Vec<RuntimeProp>);

impl<'a> Visit<'a> for RuntimeProps {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if property.key.static_name().as_deref() == Some("props")
            && let Expression::ObjectExpression(props) = &property.value
        {
            for entry in &props.properties {
                let ObjectPropertyKind::ObjectProperty(entry) = entry else {
                    panic!("runtime props must have literal fields");
                };
                let mut prop = RuntimeProp {
                    name: entry
                        .key
                        .static_name()
                        .expect("prop name")
                        .to_compact_string(),
                    constructor: None,
                    required: None,
                };
                if let Expression::ObjectExpression(fields) = &entry.value {
                    for field in &fields.properties {
                        let ObjectPropertyKind::ObjectProperty(field) = field else {
                            panic!("runtime prop options must have literal fields");
                        };
                        match (field.key.static_name().as_deref(), &field.value) {
                            (Some("type"), Expression::Identifier(value)) => {
                                prop.constructor = Some(value.name.to_compact_string());
                            }
                            (Some("required"), Expression::BooleanLiteral(value)) => {
                                prop.required = Some(value.value);
                            }
                            _ => {}
                        }
                    }
                }
                self.0.push(prop);
            }
        }
        walk::walk_object_property(self, property);
    }
}

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
    std::fs::write(path, source).expect("write fixture");
}

fn compile(root: &Path, file: &str, import: &str, destructure: bool) -> Vec<RuntimeProp> {
    let binding = if destructure { "{ bar }" } else { "props" };
    let source = cstr!(
        "<script setup lang=\"ts\">\nimport type {{ FooProps }} from '{import}';\nconst {binding} = defineProps<FooProps>();\n</script>\n<template><div/></template>"
    );
    write(root, file, &source);
    let filename = root.join(file).to_string_lossy().to_compact_string();
    let descriptor = parse_sfc(
        &source,
        SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
    )
    .expect("parse SFC");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(filename);
    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    assert_eq!(result.errors.len(), 0, "{}", result.code);
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &result.code, oxc_span::SourceType::mjs()).parse();
    assert_eq!(parsed.diagnostics.len(), 0, "{}", result.code);
    let mut props = RuntimeProps::default();
    props.visit_program(&parsed.program);
    props.0
}

fn expected() -> Vec<RuntimeProp> {
    vec![RuntimeProp {
        name: "bar".into(),
        constructor: Some("Object".into()),
        required: Some(false),
    }]
}

fn project() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("project");
    write(
        root.path(),
        "src/models/FooProps.ts",
        "export interface FooProps { bar?: Record<string, unknown> }",
    );
    write(
        root.path(),
        "app/models/FooProps.ts",
        "export interface FooProps { bar?: Record<string, unknown> }",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@lib/*":["./src/*"],"@/*":["./app/*"]}}}"#,
    );
    root
}

#[test]
fn tsconfig_paths_import_retains_declared_runtime_props() {
    let project = project();
    assert_eq!(
        compile(
            project.path(),
            "src/PathsStar.vue",
            "@lib/models/FooProps",
            false
        ),
        expected()
    );
}

#[test]
fn destructured_tsconfig_paths_import_retains_the_declared_constructor() {
    let project = project();
    assert_eq!(
        compile(
            project.path(),
            "src/Destructure.vue",
            "@lib/models/FooProps",
            true
        ),
        expected()
    );
}

#[test]
fn configured_at_alias_uses_app_layout_without_a_src_ancestor() {
    let project = project();
    assert_eq!(
        compile(project.path(), "app/Group.vue", "@/models/FooProps", false),
        expected()
    );
}

#[test]
fn relative_import_control_retains_declared_runtime_props() {
    let project = project();
    assert_eq!(
        compile(
            project.path(),
            "src/Relative.vue",
            "./models/FooProps",
            false
        ),
        expected()
    );
}

#[test]
fn generated_nuxt_config_owns_app_alias_targets() {
    let project = project();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":"./.nuxt/tsconfig.app.json"}"#,
    );
    write(
        project.path(),
        ".nuxt/tsconfig.app.json",
        r#"{"compilerOptions":{"paths":{"~/*":["../app/*"]}}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Group.vue", "~/models/FooProps", false),
        expected()
    );
}

#[test]
fn paths_edit_invalidates_a_positive_resolution_while_old_target_remains() {
    let project = project();
    assert_eq!(
        compile(
            project.path(),
            "src/Edit.vue",
            "@lib/models/FooProps",
            false
        ),
        expected()
    );
    write(
        project.path(),
        "alternate/FooProps.ts",
        "export interface FooProps { bar?: string }",
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./alternate/*"]}}}"#,
    );
    write(
        project.path(),
        "alternate/models/FooProps.ts",
        "export interface FooProps { bar?: string }",
    );
    let mut changed = expected();
    changed[0].constructor = Some("String".into());
    assert_eq!(
        compile(
            project.path(),
            "src/Edit.vue",
            "@lib/models/FooProps",
            false
        ),
        changed
    );
    assert!(project.path().join("src/models/FooProps.ts").is_file());
}

#[test]
fn exact_alias_precedes_wildcards_and_targets_keep_declared_order() {
    let project = project();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./absent/*","./src/*"],"@lib/models/*":["./absent/*","./app/models/*"],"@lib/models/FooProps":["./src/models/FooProps.ts"]}}}"#,
    );
    write(
        project.path(),
        "app/models/FooProps.ts",
        "export interface FooProps { bar?: string }",
    );
    assert_eq!(
        compile(
            project.path(),
            "app/Exact.vue",
            "@lib/models/FooProps",
            false
        ),
        expected()
    );
    assert_eq!(
        compile(
            project.path(),
            "app/Fallback.vue",
            "@lib/models/FooProps.ts",
            false
        ),
        vec![RuntimeProp {
            name: "bar".into(),
            constructor: Some("String".into()),
            required: Some(false)
        }]
    );
}

#[test]
fn earlier_paths_target_created_after_a_compile_replaces_the_old_fallback() {
    let project = project();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./first/*","./src/*"]}}}"#,
    );
    assert_eq!(
        compile(
            project.path(),
            "app/Fallback.vue",
            "@lib/models/FooProps",
            false
        ),
        expected()
    );
    write(
        project.path(),
        "first/models/FooProps.ts",
        "export interface FooProps { bar?: number }",
    );
    let mut changed = expected();
    changed[0].constructor = Some("Number".into());
    assert_eq!(
        compile(
            project.path(),
            "app/Fallback.vue",
            "@lib/models/FooProps",
            false
        ),
        changed
    );
}

#[test]
fn selected_project_snapshots_share_capture_but_isolate_overlay_resolution() {
    use std::sync::Arc;
    use vize_atelier_sfc::script::{SfcTypeProject, TypeSourceSnapshot};
    let project = project();
    let config = project.path().join("tsconfig.json");
    let first = SfcTypeProject::from_tsconfig(&config).expect("capture first project");
    for _ in 0..512 {
        assert!(Arc::ptr_eq(
            &first,
            &SfcTypeProject::from_tsconfig(&config).expect("reuse capture")
        ));
    }
    let input = project.path().join("app/Overlay.vue");
    let overlay = project.path().join("src/models/OnlyOverlay.ts");
    let overlay_identity = overlay
        .parent()
        .expect("overlay parent")
        .canonicalize()
        .expect("physical parent")
        .join("OnlyOverlay.ts");
    let sources = TypeSourceSnapshot::with_project(
        [(
            overlay.clone(),
            Arc::from("export interface Overlay { value: string }"),
        )],
        Arc::clone(&first),
    );
    assert_eq!(
        sources.resolve_import(&input, "@lib/models/OnlyOverlay"),
        Some(overlay_identity)
    );
    assert_eq!(
        sources.read(&overlay).as_deref(),
        Some("export interface Overlay { value: string }")
    );
    assert_eq!(
        TypeSourceSnapshot::default().resolve_import(&input, "@lib/models/OnlyOverlay"),
        None
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./app/*"]}}}"#,
    );
    let second = SfcTypeProject::from_tsconfig(&config).expect("capture changed project");
    assert!(!Arc::ptr_eq(&first, &second));
    let changed = TypeSourceSnapshot::with_project([], second);
    assert_eq!(
        changed.resolve_import(&input, "@lib/models/FooProps"),
        Some(
            project
                .path()
                .join("app/models/FooProps.ts")
                .canonicalize()
                .expect("physical app target")
        )
    );
    assert_eq!(
        sources.resolve_import(&input, "@lib/models/FooProps"),
        Some(
            project
                .path()
                .join("src/models/FooProps.ts")
                .canonicalize()
                .expect("physical src target")
        )
    );
}

fn compile_source(
    root: &Path,
    source: &str,
) -> Result<vize_atelier_sfc::SfcCompileResult, vize_atelier_sfc::SfcError> {
    let filename = root
        .join("app/Checked.vue")
        .to_string_lossy()
        .to_compact_string();
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
    )
    .expect("parse checked source");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(filename);
    compile_sfc(&descriptor, options)
}

#[test]
fn unresolved_imported_props_fail_instead_of_emitting_empty_runtime_props() {
    let project = project();
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type { Missing } from '@missing/Props'; defineProps<Missing>();</script>").expect_err("unresolved props must fail");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert_eq!(
        error.message,
        "Cannot resolve imported macro type `Missing` from `@missing/Props`."
    );
}

#[test]
fn unused_unresolved_type_import_does_not_block_runtime_only_compilation() {
    let project = project();
    let result = compile_source(project.path(), "<script setup lang=\"ts\">import type { Missing } from '@missing/Props'; defineProps<{ bar?: string }>();</script>").expect("unused imported type does not own props");
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn invalid_project_configuration_is_a_compile_error() {
    let project = project();
    write(project.path(), "tsconfig.json", "{ broken");
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type { FooProps } from '@lib/models/FooProps'; defineProps<FooProps>();</script>").expect_err("invalid config must fail");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert!(error.message.starts_with("invalid TypeScript project "));
}

#[test]
fn unresolved_macro_imports_are_checked_through_local_aliases_and_inheritance() {
    let project = project();
    for declaration in ["type Props = Missing", "interface Props extends Missing {}"] {
        let source = cstr!(
            "<script setup lang=\"ts\">import type {{ Missing }} from '@missing/Props'; {declaration}; defineProps<Props>();</script>"
        );
        let error = compile_source(project.path(), &source)
            .expect_err("indirect unresolved props must fail");
        assert_eq!(
            error.message,
            "Cannot resolve imported macro type `Missing` from `@missing/Props`."
        );
    }
}

#[test]
fn runtime_output_tracks_type_files_and_absent_project_selection_inputs() {
    let project = project();
    let result = compile_source(project.path(), "<script setup lang=\"ts\">import type { FooProps } from '@lib/models/FooProps'; defineProps<FooProps>();</script>").expect("compile tracked input");
    let mut expected = vec![
        project
            .path()
            .join("app/tsconfig.json")
            .to_string_lossy()
            .into_owned(),
        project
            .path()
            .join("tsconfig.json")
            .to_string_lossy()
            .into_owned(),
        project
            .path()
            .join("src/models/FooProps.ts")
            .canonicalize()
            .expect("physical type file")
            .to_string_lossy()
            .into_owned(),
    ];
    expected.sort();
    assert_eq!(
        result
            .dependencies
            .iter()
            .map(|path| path.as_str())
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|path| path.as_str())
            .collect::<Vec<_>>()
    );
}
