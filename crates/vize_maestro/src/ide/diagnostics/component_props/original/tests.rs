#![expect(
    clippy::unwrap_used,
    reason = "fixtures must open and supply complete original operations"
)]
use super::*;
use std::{fs, path::Path};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};

#[cfg(feature = "micro-profiling")]
mod profile;

fn observe<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_native(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    )
}

fn original(state: &ServerState, uri: &Url, source: &str) -> Vec<Diagnostic> {
    let descriptor = state.sfc_descriptor(uri, source).unwrap();
    DiagnosticService::collect_component_required_prop_diagnostics(
        state,
        uri,
        source,
        &descriptor,
        &LineIndex::new(source),
    )
}

fn open(state: &ServerState, uri: &Url, source: &str) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
}

fn child(dir: &Path) {
    fs::write(dir.join("Box.vue"), "<script setup lang='ts'>defineProps<{ first: string; userName: string; optional?: number }>()</script>").unwrap();
}

#[test]
fn original_required_props_keep_nested_order_and_utf16_ranges() {
    let temp = tempfile::tempdir().unwrap();
    child(temp.path());
    let source = "<script setup lang='ts'>import Box from './Box.vue'; const user='x';</script>\r\n<template>🌱<Box first><Box/><Box first :user-name='user'/></Box><Box user-name='猫'/></template>";
    let uri = Url::from_file_path(temp.path().join("Parent.vue")).unwrap();
    let state = ServerState::new();
    open(&state, &uri, source);
    let arena = Allocator::new();
    let owner = observe(&arena, source);
    let inputs = owner.setup_component_imports().unwrap();
    let previous = original(&state, &uri, source);
    let current =
        DiagnosticService::imported_component_required_props(&state, &uri, &inputs).unwrap();
    assert_eq!(current, previous);
    assert_eq!(current.len(), 3);
    assert_eq!(
        current.iter().map(|d| &d.message).collect::<Vec<_>>(),
        [
            &required_props_message("Box", &["first".into(), "userName".into()]),
            &required_props_message("Box", &["userName".into()]),
            &required_props_message("Box", &["first".into()]),
        ]
    );
    let lines = LineIndex::new(source);
    for (diag, row) in current.iter().zip([1, 0, 3]) {
        let span = inputs.sites()[row].name_span();
        let (l, c) = lines.line_col(span.start as usize);
        assert_eq!(
            diag.range,
            Range::new(Position::new(l, c), Position::new(l, c + 3))
        );
    }
    let other = Url::from_file_path(temp.path().join("Missing.vue")).unwrap();
    assert_eq!(
        DiagnosticService::imported_component_required_props(&state, &other, &inputs),
        Err(ComponentPropDiagnosticIssue::Document)
    );
    open(&state, &uri, "<template>changed</template>");
    assert_eq!(
        DiagnosticService::imported_component_required_props(&state, &uri, &inputs),
        Err(ComponentPropDiagnosticIssue::SourceRevision)
    );
}

#[test]
#[ignore = "original workspace component metadata and complete LSP diagnostics"]
fn original_workspace_diagnostics() {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let (mut count, mut sites, mut diagnostics, mut bytes) = (0, 0, 0, 0);
    let mut refused_bindings = 0;
    let mut admitted_imports = 0;
    for name in manifest.split_terminator('\0') {
        let path = root.join(name);
        let data = fs::read(&path).unwrap();
        let Ok(source) = std::str::from_utf8(&data) else {
            continue;
        };
        let arena = Allocator::new();
        let owner = observe(&arena, source);
        let Ok(inputs) = owner.setup_component_imports() else {
            continue;
        };
        admitted_imports += 1;
        let path = if path.extension().is_some_and(|e| e == "txt") {
            path.with_extension("")
        } else {
            path
        };
        let uri = Url::from_file_path(path).unwrap();
        let state = ServerState::new();
        open(&state, &uri, source);
        let actual =
            match DiagnosticService::imported_component_required_props(&state, &uri, &inputs) {
                Ok(rows) => rows,
                Err(ComponentPropDiagnosticIssue::Props(ComponentPropIssue::Binding(span))) => {
                    assert_eq!(
                        inputs.try_for_each_in_postorder(|site| site.visit_passed_props(|_| {})),
                        Err(ComponentPropIssue::Binding(span)),
                        "{name}"
                    );
                    refused_bindings += 1;
                    continue;
                }
                Err(issue) => panic!("{name}: {issue:?}"),
            };
        let old = original(&state, &uri, source);
        assert_eq!(actual, old, "{name}");
        count += 1;
        sites += inputs.sites().len();
        diagnostics += actual.len();
        bytes += serde_json::to_vec(&actual).unwrap().len();
    }
    assert_eq!(admitted_imports, count + refused_bindings);
    assert!(count > 0);
    eprintln!(
        "required_props_workspaces inputs={count} refused_bindings={refused_bindings} sites={sites} diagnostics={diagnostics} bytes={bytes}"
    );
}
