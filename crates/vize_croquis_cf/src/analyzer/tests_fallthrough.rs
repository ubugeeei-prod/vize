use super::{CrossFileAnalyzer, CrossFileOptions};
use std::path::Path;
use vize_carton::{CompactString, smallvec};
use vize_croquis::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_croquis::macros::PropDefinition;
use vize_croquis::{Croquis, ScopeId};

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
                value: None,
                start: 8,
                end: 20,
                is_dynamic: false,
            },
            PassedProp {
                name: CompactString::new("trackingId"),
                value: None,
                start: 21,
                end: 40,
                is_dynamic: true,
            }
        ],
        events: smallvec![EventListener {
            name: CompactString::new("close"),
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
