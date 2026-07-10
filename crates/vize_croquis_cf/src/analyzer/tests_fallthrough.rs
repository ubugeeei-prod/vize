use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::diagnostics::CrossFileDiagnosticKind;
use std::path::Path;
use vize_armature::parse;
use vize_carton::Bump;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

#[path = "tests_fallthrough/base.rs"]
mod base;
#[path = "tests_fallthrough/dynamic_names.rs"]
mod dynamic_names;

fn analyze_template(template: &str) -> Croquis {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(
        errors.is_empty(),
        "template should parse cleanly: {errors:?}"
    );

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    analyzer.finish()
}

fn analyze_setup_component(script: &str, template: &str) -> Croquis {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(
        errors.is_empty(),
        "template should parse cleanly: {errors:?}"
    );

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    analyzer.finish()
}

fn analyze_options_component(script: &str, template: &str) -> Croquis {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(
        errors.is_empty(),
        "template should parse cleanly: {errors:?}"
    );

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    analyzer.finish()
}

#[test]
fn parsed_declared_emit_is_not_fallthrough_but_undeclared_native_listener_is() {
    let child_template = "<main></main><aside></aside>";
    let parent_template = r#"<Child @save-item="handleSave" @click="handleNativeClick"></Child>"#;
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component(
            "const emit = defineEmits<{ (event: 'saveItem', value: string): void }>()",
            child_template,
        ),
    );
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(parent_template),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let usage = result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed Child usage should be retained");
    assert_eq!(usage.parent_file_id, parent_id);
    assert_eq!(usage.attrs.len(), 2);

    let declared = usage
        .attrs
        .iter()
        .find(|attr| attr.name == "onSaveItem")
        .expect("kebab listener should be normalized to its handler key");
    assert_eq!(
        declared.kind,
        crate::rules::FallthroughUsageAttrKind::Listener
    );
    assert!(declared.declared_event);
    assert!(!declared.declared_prop);
    assert!(!declared.fallthrough);
    assert_eq!(
        &parent_template[declared.source_start as usize..declared.source_end as usize],
        r#"@save-item="handleSave""#
    );

    let native = usage
        .attrs
        .iter()
        .find(|attr| attr.name == "onClick")
        .expect("undeclared native listener should be retained");
    assert!(!native.declared_event);
    assert!(native.standard_html_attr);
    assert!(native.fallthrough);
    assert_eq!(
        &parent_template[native.source_start as usize..native.source_end as usize],
        r#"@click="handleNativeClick""#
    );

    let child_info = result
        .fallthrough_info
        .iter()
        .find(|info| info.file_id == child_id)
        .expect("child fallthrough info should be retained");
    assert!(child_info.passed_attrs.contains("onSaveItem"));
    assert!(child_info.passed_attrs.contains("onClick"));
    assert!(!child_info.fallthrough_attrs.contains("onSaveItem"));
    assert!(child_info.fallthrough_attrs.contains("onClick"));
    assert!(child_info.declared_events.contains("saveItem"));

    let child_fact = result
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == child_id)
        .expect("child component aggregate should be retained");
    assert_eq!(child_fact.passed_attr_count, 2);
    assert_eq!(child_fact.listener_attr_count, 2);
    assert_eq!(child_fact.declared_event_attr_count, 1);
    assert_eq!(child_fact.declared_event_count, 1);
    assert_eq!(child_fact.fallthrough_attr_count, 1);
    assert_eq!(child_fact.safe_standard_fallthrough_attr_count, 1);
    assert_eq!(child_fact.risky_unconsumed_fallthrough_attr_count, 0);

    let summary = result
        .fallthrough_summary
        .expect("fallthrough summary should be populated");
    assert_eq!(summary.passed_attr_count, 2);
    assert_eq!(summary.declared_event_count, 1);
    assert_eq!(summary.undeclared_passed_attr_count, 1);

    let multi_root = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .expect("the undeclared native listener should still require explicit fallthrough");
    assert_eq!(multi_root.primary_file, child_id);
    assert_eq!(multi_root.primary_offset, 0);
    assert_eq!(multi_root.related_files.len(), 1);
    assert_eq!(multi_root.related_files[0].0, parent_id);
    assert_eq!(multi_root.related_files[0].1, native.source_start);
    assert_eq!(multi_root.related_files[0].2, "onClick passed to <Child>");
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
    )));

    let json = serde_json::to_value(usage).expect("usage fact should serialize");
    assert_eq!(json["attrs"][0]["declaredEvent"], true);
    assert_eq!(json["attrs"][0]["fallthrough"], false);
}

#[test]
fn parsed_runtime_kebab_emit_matches_camel_listener_without_diagnostics() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component(
            "const emit = defineEmits(['save-item'])",
            "<main></main><aside></aside>",
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(r#"<Child @saveItem="handleSave"></Child>"#),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let listener = &result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed Child usage should be retained")
        .attrs[0];
    assert_eq!(listener.name, "onSaveItem");
    assert!(listener.declared_event);
    assert!(!listener.fallthrough);
    assert_eq!(
        result
            .fallthrough_component_facts
            .iter()
            .find(|fact| fact.file_id == child_id)
            .expect("child aggregate")
            .fallthrough_attr_count,
        0
    );
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MultiRootMissingAttrs
            | CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
    )));
}

#[test]
fn parsed_declared_native_event_consumes_native_listener() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component(
            "const emit = defineEmits(['click'])",
            "<main></main><aside></aside>",
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(r#"<Child @click="handleClick"></Child>"#),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let listener = &result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed Child usage should be retained")
        .attrs[0];
    assert_eq!(listener.name, "onClick");
    assert!(listener.declared_event);
    assert!(!listener.fallthrough);
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MultiRootMissingAttrs
            | CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
    )));
}

#[test]
fn parsed_similar_but_undeclared_listener_remains_fallthrough() {
    let parent_template = r#"<Child @save-item="handleSave"></Child>"#;
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component(
            "const emit = defineEmits(['save'])",
            "<main></main><aside></aside>",
        ),
    );
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(parent_template),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let listener = &result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed Child usage should be retained")
        .attrs[0];
    assert_eq!(listener.name, "onSaveItem");
    assert!(!listener.declared_event);
    assert!(listener.fallthrough);

    let multi_root = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .expect("a longer undeclared event name must remain fallthrough");
    assert_eq!(multi_root.related_files.len(), 1);
    assert_eq!(multi_root.related_files[0].0, parent_id);
    assert_eq!(multi_root.related_files[0].1, listener.source_start);
    assert_eq!(
        &parent_template[listener.source_start as usize..listener.source_end as usize],
        r#"@save-item="handleSave""#
    );
}

#[test]
fn parsed_options_api_emit_is_not_fallthrough() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child = analyze_options_component(
        "export default { emits: { closeDialog: null } }",
        "<main></main><aside></aside>",
    );
    assert!(
        child
            .macros
            .emits()
            .iter()
            .any(|event| event.name == "closeDialog"),
        "Options API emits should use the existing Croquis event facts"
    );
    let child_id = analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child);
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(r#"<Child @close-dialog="close"></Child>"#),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let listener = &result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed Child usage should be retained")
        .attrs[0];
    assert_eq!(listener.name, "onCloseDialog");
    assert!(listener.declared_event);
    assert!(!listener.fallthrough);
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MultiRootMissingAttrs
            | CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
    )));
}
