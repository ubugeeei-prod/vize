use super::FallthroughInfo;
use serde::Serialize;

/// Stable counters for fallthrough attribute analysis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallthroughSummary {
    pub component_count: usize,
    pub components_with_passed_attrs: usize,
    pub components_consuming_fallthrough_attrs: usize,
    pub components_with_potential_issues: usize,
    pub inherit_attrs_disabled_count: usize,
    pub uses_attrs_count: usize,
    pub binds_attrs_count: usize,
    pub multi_root_count: usize,
    pub multi_root_without_attrs_count: usize,
    pub passed_attr_count: usize,
    pub declared_prop_count: usize,
    pub declared_event_count: usize,
    pub undeclared_passed_attr_count: usize,
    pub consumed_fallthrough_attr_count: usize,
    pub unconsumed_fallthrough_attr_count: usize,
    pub safe_standard_fallthrough_attr_count: usize,
    pub risky_unconsumed_fallthrough_attr_count: usize,
    pub max_passed_attrs: usize,
    pub max_root_element_count: usize,
}

/// Summarize fallthrough attribute facts for reports and complexity scoring.
pub fn summarize_fallthrough(infos: &[FallthroughInfo]) -> FallthroughSummary {
    let mut summary = FallthroughSummary {
        component_count: infos.len(),
        ..FallthroughSummary::default()
    };

    for info in infos {
        if !info.passed_attrs.is_empty() {
            summary.components_with_passed_attrs += 1;
        }
        if info.consumed_fallthrough_attr_count() > 0 {
            summary.components_consuming_fallthrough_attrs += 1;
        }
        if info.has_potential_issues() {
            summary.components_with_potential_issues += 1;
        }
        if info.inherit_attrs_disabled {
            summary.inherit_attrs_disabled_count += 1;
        }
        if info.uses_attrs {
            summary.uses_attrs_count += 1;
        }
        if info.binds_attrs {
            summary.binds_attrs_count += 1;
        }
        if info.root_element_count > 1 {
            summary.multi_root_count += 1;
            if !info.binds_attrs {
                summary.multi_root_without_attrs_count += 1;
            }
        }

        summary.passed_attr_count += info.passed_attrs.len();
        summary.declared_prop_count += info.declared_props.len();
        summary.declared_event_count += info.declared_events.len();
        summary.undeclared_passed_attr_count += info.fallthrough_attr_count();
        summary.consumed_fallthrough_attr_count += info.consumed_fallthrough_attr_count();
        summary.unconsumed_fallthrough_attr_count += info.unconsumed_fallthrough_attr_count();
        summary.safe_standard_fallthrough_attr_count += info.safe_standard_fallthrough_attr_count();
        summary.risky_unconsumed_fallthrough_attr_count +=
            info.risky_unconsumed_fallthrough_attr_count();
        summary.max_passed_attrs = summary.max_passed_attrs.max(info.passed_attrs.len());
        summary.max_root_element_count =
            summary.max_root_element_count.max(info.root_element_count);
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FileId;
    use vize_carton::{CompactString, FxHashSet};

    fn set(names: &[&str]) -> FxHashSet<CompactString> {
        names.iter().map(|name| CompactString::new(*name)).collect()
    }

    #[test]
    fn summary_counts_fallthrough_risk_dimensions() {
        let infos = vec![
            FallthroughInfo {
                file_id: FileId::new(1),
                inherit_attrs_disabled: false,
                uses_attrs: false,
                binds_attrs: false,
                root_element_count: 2,
                passed_attrs: set(&["title", "kind"]),
                fallthrough_attrs: set(&["title"]),
                dynamic_name_fallthrough_attrs: FxHashSet::default(),
                declared_props: set(&["kind"]),
                declared_events: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
            FallthroughInfo {
                file_id: FileId::new(2),
                inherit_attrs_disabled: true,
                uses_attrs: true,
                binds_attrs: false,
                root_element_count: 1,
                passed_attrs: set(&["class"]),
                fallthrough_attrs: set(&["class"]),
                dynamic_name_fallthrough_attrs: FxHashSet::default(),
                declared_props: FxHashSet::default(),
                declared_events: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
            FallthroughInfo {
                file_id: FileId::new(3),
                inherit_attrs_disabled: false,
                uses_attrs: false,
                binds_attrs: true,
                root_element_count: 3,
                passed_attrs: FxHashSet::default(),
                fallthrough_attrs: FxHashSet::default(),
                dynamic_name_fallthrough_attrs: FxHashSet::default(),
                declared_props: FxHashSet::default(),
                declared_events: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
        ];

        let summary = summarize_fallthrough(&infos);

        assert_eq!(summary.component_count, 3);
        assert_eq!(summary.components_with_passed_attrs, 2);
        assert_eq!(summary.components_consuming_fallthrough_attrs, 1);
        assert_eq!(summary.components_with_potential_issues, 1);
        assert_eq!(summary.inherit_attrs_disabled_count, 1);
        assert_eq!(summary.uses_attrs_count, 1);
        assert_eq!(summary.binds_attrs_count, 1);
        assert_eq!(summary.multi_root_count, 2);
        assert_eq!(summary.multi_root_without_attrs_count, 1);
        assert_eq!(summary.passed_attr_count, 3);
        assert_eq!(summary.declared_prop_count, 1);
        assert_eq!(summary.declared_event_count, 0);
        assert_eq!(summary.undeclared_passed_attr_count, 2);
        assert_eq!(summary.consumed_fallthrough_attr_count, 1);
        assert_eq!(summary.unconsumed_fallthrough_attr_count, 1);
        assert_eq!(summary.safe_standard_fallthrough_attr_count, 2);
        assert_eq!(summary.risky_unconsumed_fallthrough_attr_count, 0);
        assert_eq!(summary.max_passed_attrs, 2);
        assert_eq!(summary.max_root_element_count, 3);

        let json = serde_json::to_value(summary).unwrap();
        assert_eq!(json["componentCount"], 3);
        assert_eq!(json["componentsWithPotentialIssues"], 1);
        assert_eq!(json["declaredEventCount"], 0);
        assert_eq!(json["safeStandardFallthroughAttrCount"], 2);
    }

    #[test]
    fn summary_distinguishes_consumed_safe_and_risky_attrs() {
        let infos = vec![
            FallthroughInfo {
                file_id: FileId::new(1),
                inherit_attrs_disabled: false,
                uses_attrs: false,
                binds_attrs: false,
                root_element_count: 1,
                passed_attrs: set(&["class", "data-testid", "onClick"]),
                fallthrough_attrs: set(&["class", "data-testid", "onClick"]),
                dynamic_name_fallthrough_attrs: FxHashSet::default(),
                declared_props: FxHashSet::default(),
                declared_events: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
            FallthroughInfo {
                file_id: FileId::new(2),
                inherit_attrs_disabled: false,
                uses_attrs: false,
                binds_attrs: false,
                root_element_count: 2,
                passed_attrs: set(&["aria-label", "trackingId"]),
                fallthrough_attrs: set(&["aria-label", "trackingId"]),
                dynamic_name_fallthrough_attrs: FxHashSet::default(),
                declared_props: FxHashSet::default(),
                declared_events: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
        ];

        let summary = summarize_fallthrough(&infos);

        assert_eq!(summary.components_consuming_fallthrough_attrs, 1);
        assert_eq!(summary.undeclared_passed_attr_count, 5);
        assert_eq!(summary.consumed_fallthrough_attr_count, 3);
        assert_eq!(summary.unconsumed_fallthrough_attr_count, 2);
        assert_eq!(summary.safe_standard_fallthrough_attr_count, 4);
        assert_eq!(summary.risky_unconsumed_fallthrough_attr_count, 1);
    }

    #[test]
    fn summary_excludes_declared_event_listeners_from_fallthrough_counts() {
        let infos = vec![FallthroughInfo {
            file_id: FileId::new(1),
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 2,
            passed_attrs: set(&["onSaveItem", "onClick"]),
            fallthrough_attrs: set(&["onClick"]),
            dynamic_name_fallthrough_attrs: FxHashSet::default(),
            declared_props: FxHashSet::default(),
            declared_events: set(&["save-item"]),
            template_start: 0,
            template_end: 10,
        }];

        let summary = summarize_fallthrough(&infos);

        assert_eq!(summary.passed_attr_count, 2);
        assert_eq!(summary.declared_event_count, 1);
        assert_eq!(summary.undeclared_passed_attr_count, 1);
        assert_eq!(summary.unconsumed_fallthrough_attr_count, 1);
        assert_eq!(summary.safe_standard_fallthrough_attr_count, 1);
        assert_eq!(summary.risky_unconsumed_fallthrough_attr_count, 0);
    }
}
