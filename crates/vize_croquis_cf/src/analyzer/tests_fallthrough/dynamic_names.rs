use super::*;
use crate::analyzer::CrossFileResult;
use serde_json::{Value, json};

const CHILD_SCRIPT: &str = concat!(
    "defineProps<{ kind?: string; class?: string; modelValue?: string }>();\n",
    "defineEmits(['save', 'click', 'update:modelValue']);",
);
const MULTI_ROOT_CHILD: &str = "<main></main><aside></aside>";
const DYNAMIC_PARENT: &str = concat!(
    "<p>雪</p><Child :kind=\"kind\" :[kind]=\"maybeKind\" ",
    ":[class]=\"maybeClass\" v-model:[modelValue]=\"model\" ",
    "@save=\"save\" @[click]=\"dynamicClick\" v-bind=\"$attrs\" />",
);

fn analyze_dynamic_pair(parent_first: bool) -> (CrossFileResult, crate::FileId, crate::FileId) {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let parent = analyze_template(DYNAMIC_PARENT);
    let child = analyze_setup_component(CHILD_SCRIPT, MULTI_ROOT_CHILD);

    let (child_id, parent_id) = if parent_first {
        let parent_id = analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent);
        let child_id = analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child);
        (child_id, parent_id)
    } else {
        let child_id = analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child);
        let parent_id = analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent);
        (child_id, parent_id)
    };
    analyzer.rebuild_component_edges();
    (analyzer.analyze(), child_id, parent_id)
}

fn source_range(needle: &str) -> (u32, u32) {
    let start = DYNAMIC_PARENT.find(needle).unwrap() as u32;
    (start, start + needle.len() as u32)
}

fn attr_json(
    name: &str,
    kind: &str,
    source: &str,
    name_is_dynamic: bool,
    declared_prop: bool,
    declared_event: bool,
    standard_html_attr: bool,
) -> Value {
    let (source_start, source_end) = source_range(source);
    json!({
        "name": name,
        "kind": kind,
        "sourceStart": source_start,
        "sourceEnd": source_end,
        "nameIsDynamic": name_is_dynamic,
        "dynamic": true,
        "declaredProp": declared_prop,
        "declaredEvent": declared_event,
        "standardHtmlAttr": standard_html_attr,
        "fallthrough": !declared_prop && !declared_event
    })
}

#[test]
fn parsed_dynamic_names_never_match_static_contracts() {
    let (result, child_id, parent_id) = analyze_dynamic_pair(false);
    let usage = result
        .fallthrough_usage_facts
        .iter()
        .find(|fact| fact.child_file_id == child_id)
        .unwrap();

    assert_eq!(usage.parent_file_id, parent_id);
    assert!(usage.has_spread_attrs);
    let attrs = serde_json::to_value(&usage.attrs).unwrap();
    assert_eq!(
        attrs,
        json!([
            attr_json("kind", "prop", ":kind=\"kind\"", false, true, false, false),
            attr_json(
                "kind",
                "prop",
                ":[kind]=\"maybeKind\"",
                true,
                false,
                false,
                false
            ),
            attr_json(
                "class",
                "prop",
                ":[class]=\"maybeClass\"",
                true,
                false,
                false,
                false
            ),
            attr_json(
                "modelValue",
                "prop",
                "v-model:[modelValue]=\"model\"",
                true,
                false,
                false,
                false
            ),
            attr_json(
                "onUpdate:modelValue",
                "listener",
                "v-model:[modelValue]=\"model\"",
                true,
                false,
                false,
                false
            ),
            attr_json(
                "onSave",
                "listener",
                "@save=\"save\"",
                false,
                false,
                true,
                true
            ),
            attr_json(
                "onClick",
                "listener",
                "@[click]=\"dynamicClick\"",
                true,
                false,
                false,
                false
            )
        ])
    );

    for attr in usage.attrs.iter().filter(|attr| attr.name_is_dynamic) {
        assert!(!attr.declared_prop);
        assert!(!attr.declared_event);
        assert!(!attr.standard_html_attr);
        assert!(attr.fallthrough);
        assert_eq!(
            &DYNAMIC_PARENT[attr.source_start as usize..attr.source_end as usize],
            match attr.name.as_str() {
                "kind" => ":[kind]=\"maybeKind\"",
                "class" => ":[class]=\"maybeClass\"",
                "modelValue" | "onUpdate:modelValue" => "v-model:[modelValue]=\"model\"",
                "onClick" => "@[click]=\"dynamicClick\"",
                other => panic!("unexpected dynamic attr {other}"),
            }
        );
    }

    let info = result
        .fallthrough_info
        .iter()
        .find(|info| info.file_id == child_id)
        .unwrap();
    assert_eq!(info.dynamic_name_fallthrough_attrs.len(), 5);
    assert!(info.dynamic_name_fallthrough_attrs.contains("class"));
    assert!(info.dynamic_name_fallthrough_attrs.contains("onClick"));
    assert_eq!(info.safe_standard_fallthrough_attr_count(), 0);
    assert_eq!(info.risky_unconsumed_fallthrough_attr_count(), 5);

    let component = result
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == child_id)
        .unwrap();
    assert_eq!(component.usage_attr_count, 7);
    assert_eq!(component.dynamic_name_attr_count, 5);
    assert_eq!(component.declared_prop_attr_count, 1);
    assert_eq!(component.declared_event_attr_count, 1);
    assert_eq!(component.safe_standard_fallthrough_attr_count, 0);
    assert_eq!(component.risky_unconsumed_fallthrough_attr_count, 5);

    let multi_root = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .unwrap();
    assert_eq!(multi_root.related_files.len(), 6);
    assert!(multi_root.related_files.iter().any(|related| {
        related.0 == parent_id
            && related.1 == source_range(":[class]=\"maybeClass\"").0
            && related.2 == "class passed to <Child>"
    }));
}

#[test]
fn dynamic_attr_order_is_independent_of_file_registration_order() {
    let (first, first_child, _) = analyze_dynamic_pair(false);
    let (second, second_child, _) = analyze_dynamic_pair(true);
    let attrs_for = |result: &CrossFileResult, child| {
        result
            .fallthrough_usage_facts
            .iter()
            .find(|fact| fact.child_file_id == child)
            .map(|fact| serde_json::to_value(&fact.attrs).unwrap())
            .unwrap()
    };

    assert_eq!(
        attrs_for(&first, first_child),
        attrs_for(&second, second_child)
    );
}

fn analyze_boundary(child_script: &str, child_template: &str) -> (CrossFileResult, crate::FileId) {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component(child_script, child_template),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template("<Child :[class]=\"value\" />"),
    );
    analyzer.rebuild_component_edges();
    (analyzer.analyze(), child_id)
}

#[test]
fn dynamic_name_risk_respects_consumption_boundaries() {
    let (single, single_id) = analyze_boundary("", "<main></main>");
    let single_fact = single
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == single_id)
        .unwrap();
    assert_eq!(single_fact.consumed_fallthrough_attr_count, 1);
    assert_eq!(single_fact.risky_unconsumed_fallthrough_attr_count, 0);
    assert!(single.diagnostics.is_empty());

    let (explicit, explicit_id) =
        analyze_boundary("", "<main v-bind=\"$attrs\"></main><aside></aside>");
    let explicit_fact = explicit
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == explicit_id)
        .unwrap();
    assert!(explicit_fact.binds_attrs);
    assert_eq!(explicit_fact.consumed_fallthrough_attr_count, 1);
    assert_eq!(explicit_fact.risky_unconsumed_fallthrough_attr_count, 0);
    assert!(explicit.diagnostics.is_empty());

    let (disabled, disabled_id) =
        analyze_boundary("defineOptions({ inheritAttrs: false })", "<main></main>");
    let disabled_fact = disabled
        .fallthrough_component_facts
        .iter()
        .find(|fact| fact.file_id == disabled_id)
        .unwrap();
    assert!(disabled_fact.inherit_attrs_disabled);
    assert_eq!(disabled_fact.unconsumed_fallthrough_attr_count, 1);
    assert_eq!(disabled_fact.risky_unconsumed_fallthrough_attr_count, 1);
    assert!(disabled.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::InheritAttrsDisabledUnused
    )));
}

#[test]
fn dynamic_names_do_not_trigger_exact_prop_or_emit_diagnostics() {
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::default()
            .with_props_validation(true)
            .with_component_emits(true),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        analyze_setup_component("defineProps<{ kind: string }>()", "<main></main>"),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        analyze_template("<Child :[kind]=\"value\" @[ghost]=\"handler\" />"),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(result.props_validation_issues.is_empty());
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MissingRequiredProp { .. }
            | CrossFileDiagnosticKind::UndeclaredProp { .. }
            | CrossFileDiagnosticKind::PropTypeMismatch { .. }
            | CrossFileDiagnosticKind::UnmatchedEventListener { .. }
    )));
}
