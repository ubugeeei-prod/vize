//! The pinned project is physically consumed by the real host and compiler.

use std::path::Path;
use std::sync::Arc;

use oxc_ast::ast::{Expression, ObjectProperty, ObjectPropertyKind};
use oxc_ast_visit::{Visit, walk};
use serde_json::Value;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, compile_sfc_for_adapter_with_project, parse_sfc,
};
use vize_l0::ToCompactString;

use super::*;

const FIXTURE: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/imported-project-ownership-11794/case.json"
);

#[derive(Default)]
struct Props(Vec<Value>);

impl<'a> Visit<'a> for Props {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if property.key.static_name().as_deref() == Some("props") {
            let Expression::ObjectExpression(object) = &property.value else {
                unreachable!("literal props object required")
            };
            let mut props = serde_json::Map::new();
            for prop in &object.properties {
                let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                    unreachable!("literal prop required")
                };
                let Expression::ObjectExpression(options) = &prop.value else {
                    unreachable!("literal prop options required")
                };
                let mut fields = serde_json::Map::new();
                for option in &options.properties {
                    let ObjectPropertyKind::ObjectProperty(option) = option else {
                        unreachable!("literal option required")
                    };
                    let value = match &option.value {
                        Expression::Identifier(identifier) => Value::from(identifier.name.as_str()),
                        Expression::BooleanLiteral(boolean) => Value::from(boolean.value),
                        _ => unreachable!("constructor and required boolean expected"),
                    };
                    fields.insert(
                        option.key.static_name().expect("option name").into_owned(),
                        value,
                    );
                }
                props.insert(
                    prop.key.static_name().expect("prop name").into_owned(),
                    Value::Object(fields),
                );
            }
            self.0.push(Value::Object(props));
        }
        walk::walk_object_property(self, property);
    }
}

fn props(code: &str) -> Value {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut props = Props::default();
    props.visit_program(&parsed.program);
    assert_eq!(props.0.len(), 1);
    props.0.remove(0)
}

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create input directory");
    std::fs::write(path, source).expect("write input");
}

fn materialize(root: &Path) -> Value {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("pinned JSON");
    assert_eq!(fixture["version"], 1);
    assert_eq!(fixture["issue"], 11794);
    write_files(root, &fixture["files"]);
    fixture
}

fn write_files(root: &Path, files: &Value) {
    for (file, value) in files.as_object().expect("finite inputs") {
        let source = value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(value).expect("configuration"));
        write(root, file, &source);
    }
}

fn compile(
    root: &Path,
    fixture: &Value,
    projects: &ImportedTypeProjects,
) -> Result<vize_atelier_sfc::SfcCompileResult, SfcError> {
    let path = root.join(fixture["source"].as_str().expect("source path"));
    let source = std::fs::read_to_string(&path).expect("source input");
    let filename = path.to_string_lossy().to_compact_string();
    let descriptor = parse_sfc(
        &source,
        SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
    )
    .expect("parse source");
    let selected = projects.for_descriptor(&descriptor, &path)?;
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(filename);
    let result = compile_sfc_for_adapter_with_project(
        &descriptor,
        options,
        Default::default(),
        Default::default(),
        Default::default(),
        vize_atelier_sfc::SfcScriptOutputMode::InlineTemplate,
        Default::default(),
        false,
        selected.as_deref(),
    )?;
    Ok(selected.track(result))
}

#[test]
fn pinned_reference_project_is_selected_once_and_configuration_changes_reach_compilation() {
    let project = tempfile::tempdir().expect("finite project");
    let root = project.path().canonicalize().expect("physical root");
    let fixture = materialize(&root);
    let projects = ImportedTypeProjects::default();
    let snapshot = projects
        .snapshot(&root.join("tsconfig.json"))
        .expect("capture reference graph");
    assert_eq!(
        snapshot.select(
            &root.join(fixture["source"].as_str().unwrap()),
            TsconfigSourceKind::Typed
        ),
        root.join(fixture["selectedConfig"].as_str().unwrap())
    );
    for _ in 0..512 {
        assert!(Arc::ptr_eq(
            &snapshot,
            &projects
                .snapshot(&root.join("tsconfig.json"))
                .expect("reuse project capture")
        ));
    }
    let first = compile(&root, &fixture, &projects).expect("compile owning project");
    assert_eq!(props(&first.code), fixture["expectedProps"]);
    for file in [
        "tsconfig.json",
        "projects/app/tsconfig.json",
        "app/types/FooProps.ts",
    ] {
        assert!(
            first
                .dependencies
                .iter()
                .any(|path| Path::new(path) == root.join(file)),
            "missing tracked input: {file}"
        );
    }
    let unrelated = root.join("projects/unrelated/tsconfig.json");
    assert!(
        !first
            .dependencies
            .iter()
            .any(|path| Path::new(path) == unrelated)
    );
    assert_eq!(
        type_project_owners(&unrelated),
        vec![root.join("tsconfig.json")]
    );
    let inputs =
        type_project_inputs(&snapshot.root, snapshot.revision, None).expect("shared watch inputs");
    assert!(inputs.iter().any(|path| path == &unrelated));
    write_files(&root, &fixture["configEdit"]["files"]);
    let changed = ImportedTypeProjects::default();
    let fresh = changed
        .snapshot(&root.join("tsconfig.json"))
        .expect("recapture changed ownership input");
    assert!(!Arc::ptr_eq(&snapshot, &fresh));
    let second = compile(&root, &fixture, &changed).expect("compile changed project");
    assert_eq!(props(&second.code), fixture["configEdit"]["expectedProps"]);

    write_files(&root, &fixture["ownershipEdit"]["files"]);
    let ambiguous = ImportedTypeProjects::default();
    let latest = ambiguous
        .snapshot(&root.join("tsconfig.json"))
        .expect("recapture changed membership");
    assert!(!Arc::ptr_eq(&fresh, &latest));
    assert_eq!(
        latest.select(
            &root.join(fixture["source"].as_str().unwrap()),
            TsconfigSourceKind::Typed
        ),
        root.join(fixture["ownershipEdit"]["selectedConfig"].as_str().unwrap())
    );
    let error = compile(&root, &fixture, &ambiguous).expect_err("ambiguous ownership fails closed");
    assert_eq!(
        error.message.as_str(),
        fixture["ownershipEdit"]["expectedError"].as_str().unwrap()
    );
}

#[test]
fn configuration_reverse_edges_follow_current_and_inflight_captures() {
    let project = tempfile::tempdir().expect("project");
    let root = project.path().canonicalize().expect("physical root");
    let root_config = root.join("tsconfig.json");
    write(
        &root,
        "old/tsconfig.json",
        r#"{"include":["../app/**/*.vue"]}"#,
    );
    write(
        &root,
        "new/tsconfig.json",
        r#"{"include":["../app/**/*.vue"]}"#,
    );
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./old"}]}"#,
    );
    let old_session = ImportedTypeProjects::default();
    let old = old_session.snapshot(&root_config).expect("old graph");
    assert_eq!(
        type_project_owners(&root.join("old/tsconfig.json")),
        vec![root_config.clone()]
    );
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./new"}]}"#,
    );
    let new_session = ImportedTypeProjects::default();
    let new = new_session.snapshot(&root_config).expect("new graph");
    assert_ne!(old.revision, new.revision);
    assert_eq!(
        type_project_owners(&root.join("old/tsconfig.json")),
        vec![root_config.clone()],
        "old in-flight readers retain their routing edges"
    );
    assert_eq!(
        type_project_owners(&root.join("new/tsconfig.json")),
        vec![root_config.clone()]
    );
    assert_eq!(type_project_inputs(&root_config, old.revision, None), None);
    assert_eq!(
        type_project_inputs(&root_config, new.revision, Some(new.revision)),
        Some(Vec::new())
    );
    drop(old_session);
    drop(old);
    assert_eq!(
        type_project_owners(&root.join("old/tsconfig.json")),
        Vec::<std::path::PathBuf>::new()
    );
    assert_eq!(type_project_owners(&root_config), vec![root_config]);
}

#[test]
fn watch_admission_observes_absent_parent_creation_and_config_deletion() {
    let project = tempfile::tempdir().expect("project");
    let root = project.path().canonicalize().expect("physical root");
    let config = root.join("tsconfig.json");
    let parent = root.join("parent.json");
    write(&root, "tsconfig.json", r#"{"extends":"./parent.json"}"#);
    let first_session = ImportedTypeProjects::default();
    let first = first_session.snapshot(&config).expect("capture absence");
    assert_eq!(type_project_owners(&parent), vec![config.clone()]);
    write(
        &root,
        "parent.json",
        r#"{"compilerOptions":{"strict":true}}"#,
    );
    assert_eq!(
        type_project_inputs(&config, first.revision, None),
        None,
        "newly created config must reject stale watch admission"
    );
    let second_session = ImportedTypeProjects::default();
    let second = second_session.snapshot(&config).expect("capture parent");
    assert_ne!(first.revision, second.revision);
    assert!(type_project_inputs(&config, second.revision, None).is_some());
    std::fs::remove_file(&parent).expect("delete parent");
    assert_eq!(type_project_inputs(&config, second.revision, None), None);
    let third_session = ImportedTypeProjects::default();
    let third = third_session.snapshot(&config).expect("capture deletion");
    assert_ne!(second.revision, third.revision);
    assert_eq!(type_project_owners(&parent), vec![config]);
}

#[test]
fn selected_options_are_built_once_from_the_captured_bytes_after_config_deletion() {
    let project = tempfile::tempdir().expect("project");
    let root = project.path().canonicalize().expect("physical root");
    let config = root.join("tsconfig.json");
    let source = root.join("App.vue");
    write(
        &root,
        "tsconfig.json",
        r#"{"compilerOptions":{"paths":{"@lib/*":["./types/*"]}}}"#,
    );
    write(
        &root,
        "types/FooProps.ts",
        "export interface FooProps { bar?: string }",
    );
    write(&root, "App.vue", "<template><div/></template>");
    let session = ImportedTypeProjects::default();
    let capture = session.snapshot(&config).expect("capture config bytes");
    std::fs::remove_file(&config).expect("remove live config");
    let selected = capture.select(&source, TsconfigSourceKind::Typed);
    let first = capture
        .project(&selected)
        .expect("derive from frozen bytes without config IO");
    for _ in 0..512 {
        assert!(Arc::ptr_eq(
            &first,
            &capture.project(&selected).expect("reuse options")
        ));
    }
    let sources = vize_atelier_sfc::script::TypeSourceSnapshot::with_project([], first);
    assert_eq!(
        sources.resolve_import(&source, "@lib/FooProps"),
        Some(root.join("types/FooProps.ts"))
    );
    assert_eq!(
        type_project_inputs(&config, capture.revision, None),
        None,
        "old readers remain coherent, but stale watch admission must fail"
    );
}
