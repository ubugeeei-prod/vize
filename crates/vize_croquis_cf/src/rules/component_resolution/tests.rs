use super::{
    ComponentResolutionIssueKind, analyze_component_resolution, is_builtin_component,
    is_custom_element_tag,
};
use crate::CrossFileDiagnosticKind;
use crate::graph::DependencyGraph;
use crate::registry::ModuleRegistry;
use vize_carton::{CompactString, smallvec};
use vize_croquis::analysis::ComponentUsage;
use vize_croquis::{Croquis, ScopeId};

#[test]
fn test_is_builtin_component() {
    assert!(is_builtin_component("Transition"));
    assert!(is_builtin_component("transition"));
    assert!(is_builtin_component("transition-group"));
    assert!(is_builtin_component("KeepAlive"));
    assert!(is_builtin_component("keep-alive"));
    assert!(is_builtin_component("RouterView"));
    assert!(is_builtin_component("router-view"));
    assert!(is_builtin_component("NuxtPage"));
    assert!(is_builtin_component("nuxt-page"));
    assert!(is_builtin_component("nuxt-link"));
    assert!(is_builtin_component("client-only"));
    assert!(is_builtin_component("slot"));
    assert!(!is_builtin_component("MyComponent"));
    assert!(!is_builtin_component("UserCard"));
}

#[test]
fn test_is_custom_element_tag() {
    assert!(is_custom_element_tag("my-widget"));
    assert!(is_custom_element_tag("ion-button"));
    assert!(is_custom_element_tag("sl-icon2"));
    assert!(is_custom_element_tag("my_widget-button"));
    assert!(!is_custom_element_tag("MyWidget"));
    assert!(!is_custom_element_tag("myWidget"));
    assert!(!is_custom_element_tag("ChildWidget"));
    assert!(!is_custom_element_tag("font-face"));
    assert!(!is_custom_element_tag("div"));
}

#[test]
fn unregistered_component_uses_template_usage_offset() {
    let mut registry = ModuleRegistry::new();
    let graph = DependencyGraph::new();
    let mut analysis = Croquis::new();

    analysis
        .used_components
        .insert(CompactString::new("UnknownThing"));
    analysis.component_usages.push(ComponentUsage {
        name: CompactString::new("UnknownThing"),
        start: 12,
        end: 27,
        props: smallvec![],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });

    let (file_id, _) = registry.register("Parent.vue", "", analysis);
    let (issues, diagnostics) = analyze_component_resolution(&registry, &graph);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].file_id, file_id);
    assert_eq!(
        issues[0].kind,
        ComponentResolutionIssueKind::UnregisteredComponent
    );
    assert_eq!(issues[0].offset, 12);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].primary_file, file_id);
    assert_eq!(diagnostics[0].primary_offset, 12);
    assert_eq!(
        diagnostics[0].kind,
        CrossFileDiagnosticKind::UnregisteredComponent {
            component_name: CompactString::new("UnknownThing"),
            template_offset: 12,
        }
    );
}
