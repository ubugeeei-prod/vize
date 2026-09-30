use super::imports::extract_vue_imports;
use super::losses;
use super::types::{InternalIssue, ReactivityIssueKind};
use vize_carton::FxHashSet;
use vize_croquis::reactivity::ReactiveKind;

#[inline]
pub(super) fn analyze_component_reactivity(analysis: &vize_croquis::Croquis) -> Vec<InternalIssue> {
    let mut issues = Vec::new();

    // Track which identifiers come from 'vue' imports (ref, reactive, toRefs, etc.)
    let vue_imports = extract_vue_imports(analysis);

    // Check for toRefs usage - this is the correct pattern, no warning needed
    // Check for reactive sources that indicate proper usage
    let sources = vize_croquis::facts::reactivity_sources(analysis);
    let torefs_sources: FxHashSet<&str> = sources
        .iter()
        .filter(|s| matches!(s.kind, ReactiveKind::ToRef | ReactiveKind::ToRefs))
        .map(|s| s.name.as_str())
        .collect();

    // Build a set of all reactive sources (from vue imports)
    let _reactive_sources: FxHashSet<&str> = sources.iter().map(|s| s.name.as_str()).collect();

    // Track props defined via defineProps
    let props: FxHashSet<&str> = analysis
        .macros
        .props()
        .iter()
        .map(|p| p.name.as_str())
        .collect();

    // Check if props are properly wrapped with toRef/toRefs when destructured
    if let Some(props_destructure) = analysis.macros.props_destructure() {
        for key in props_destructure.bindings.keys() {
            // Check if this destructured prop has a corresponding toRef
            if !torefs_sources.contains(key.as_str()) {
                // This prop is destructured without toRefs - Vue handles this with
                // reactive props destructure transform, so this is actually OK in modern Vue
                // We don't warn here as it's handled by the compiler
            }
        }
    }

    losses::append_reactivity_losses(analysis, &mut issues);

    // Report if vue imports are present but not used properly
    if !vue_imports.is_empty() {
        // Check if reactive sources are actually used
        for source in &sources {
            // Verify the reactive function was imported from 'vue'
            let function_name = match source.kind {
                ReactiveKind::Ref => "ref",
                ReactiveKind::ShallowRef => "shallowRef",
                ReactiveKind::Reactive => "reactive",
                ReactiveKind::ShallowReactive => "shallowReactive",
                ReactiveKind::Computed => "computed",
                ReactiveKind::Readonly => "readonly",
                ReactiveKind::ShallowReadonly => "shallowReadonly",
                ReactiveKind::ToRef => "toRef",
                ReactiveKind::ToRefs => "toRefs",
            };

            // Verify it comes from vue
            if !vue_imports.contains(function_name) {
                // The reactive function might be a local implementation or from another library
                // This is a potential issue but not necessarily an error
            }
        }
    }

    // Check for prop passed to ref() which creates a copy
    for source in &sources {
        if source.kind == ReactiveKind::Ref {
            // Check if this ref is initialized with a prop
            if props.contains(source.name.as_str()) {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::PropPassedToRef {
                        prop_name: source.name.clone(),
                    },
                    offset: source.declaration_offset,
                    end_offset: None,
                    source: Some(source.name.clone()),
                });
            }
        }
    }

    issues
}
