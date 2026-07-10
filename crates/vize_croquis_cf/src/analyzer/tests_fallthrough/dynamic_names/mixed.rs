use super::*;
use vize_carton::{CompactString, cstr};

#[test]
fn mixed_static_and_runtime_names_keep_occurrence_provenance() {
    let parent_source = "<Child class=\"fixed\" :[class]=\"dynamicValue\" />";
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component("", MULTI_ROOT_CHILD),
    );
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(parent_source),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let usage = result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .unwrap();
    assert_eq!(usage.attrs.len(), 2);
    assert!(usage.attrs[0].standard_html_attr);
    assert!(!usage.attrs[0].name_is_dynamic);
    assert!(!usage.attrs[1].standard_html_attr);
    assert!(usage.attrs[1].name_is_dynamic);

    let info = result
        .fallthrough_info
        .iter()
        .find(|info| info.file_id == child_id)
        .unwrap();
    assert_eq!(info.fallthrough_attrs.len(), 1);
    assert!(info.static_name_fallthrough_attrs.contains("class"));
    assert!(info.dynamic_name_fallthrough_attrs.contains("class"));
    assert_eq!(info.fallthrough_attr_count(), 2);
    assert_eq!(info.safe_standard_fallthrough_attr_count(), 1);
    assert_eq!(info.risky_unconsumed_fallthrough_attr_count(), 1);

    let component = result
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == child_id)
        .unwrap();
    assert_eq!(component.usage_attr_count, 2);
    assert_eq!(component.dynamic_name_attr_count, 1);
    assert_eq!(component.fallthrough_attr_count, 2);
    assert_eq!(component.safe_standard_fallthrough_attr_count, 1);
    assert_eq!(component.risky_unconsumed_fallthrough_attr_count, 1);

    let unused = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
            )
        })
        .unwrap();
    let CrossFileDiagnosticKind::UnusedFallthroughAttrs { passed_attrs } = &unused.kind else {
        unreachable!();
    };
    assert_eq!(passed_attrs, &[CompactString::new("[class]")]);
    let dynamic_start = parent_source.find(":[class]").unwrap() as u32;
    let static_start = parent_source.find("class=\"fixed\"").unwrap() as u32;
    assert_eq!(
        unused.related_files,
        [(parent_id, dynamic_start, cstr!("[class] passed to <Child>"))]
    );
    assert!(
        unused
            .related_files
            .iter()
            .all(|related| related.1 != static_start)
    );
}
