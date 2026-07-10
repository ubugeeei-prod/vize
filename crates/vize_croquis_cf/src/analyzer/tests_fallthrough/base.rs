use super::*;
use vize_carton::{CompactString, smallvec};
use vize_croquis::ScopeId;
use vize_croquis::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_croquis::macros::PropDefinition;

#[test]
fn fallthrough_component_facts_are_populated_from_analyzer() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let mut child = Croquis::new();
    child.template_info.root_element_count = 2;
    child.macros.add_prop(PropDefinition {
        name: CompactString::new("kind"),
        prop_type: None,
        required: false,
        default_value: None,
    });
    let child_id = analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child);

    let mut parent = Croquis::new();
    parent.used_components.insert(CompactString::new("Child"));
    parent.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 0,
        end: 80,
        props: smallvec![
            PassedProp {
                name: CompactString::new("kind"),
                name_is_dynamic: false,
                value: None,
                start: 8,
                end: 20,
                is_dynamic: false,
            },
            PassedProp {
                name: CompactString::new("trackingId"),
                name_is_dynamic: false,
                value: None,
                start: 21,
                end: 40,
                is_dynamic: true,
            }
        ],
        events: smallvec![EventListener {
            name: CompactString::new("close"),
            name_is_dynamic: false,
            handler: None,
            modifiers: smallvec![],
            start: 41,
            end: 55,
        }],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent);

    let result = analyzer.analyze();
    let fact = result
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == child_id)
        .expect("child component fact should be present");

    assert_eq!(fact.usage_count, 1);
    assert_eq!(fact.parent_count, 1);
    assert_eq!(fact.prop_attr_count, 2);
    assert_eq!(fact.listener_attr_count, 1);
    assert_eq!(fact.declared_prop_attr_count, 1);
    assert_eq!(fact.fallthrough_attr_count, 2);
    assert_eq!(fact.risky_unconsumed_fallthrough_attr_count, 1);
    assert!(fact.has_potential_issues);
}

#[test]
fn parsed_spread_attrs_report_multi_root_child() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_template("<main></main><aside></aside>"),
    );
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(r#"<Child v-bind="attrs" />"#),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let usage = result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .expect("parsed component usage should be retained");
    assert_eq!(usage.parent_file_id, parent_id);
    assert!(usage.has_spread_attrs);
    assert!(usage.attrs.is_empty());

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .expect("spread attrs should report the multi-root child");
    assert_eq!(diagnostic.primary_file, child_id);
    assert_eq!(diagnostic.related_files.len(), 1);
    assert_eq!(diagnostic.related_files[0].0, parent_id);
    assert_eq!(diagnostic.related_files[0].1, usage.usage_start);
}

#[test]
fn parsed_spread_attrs_allow_explicit_attrs_binding() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_template(r#"<main v-bind="$attrs"></main><aside></aside>"#),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template(r#"<Child v-bind="attrs" />"#),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert_eq!(result.fallthrough_usage_facts.len(), 1);
    assert!(result.fallthrough_usage_facts[0].has_spread_attrs);
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MultiRootMissingAttrs
    )));
}
