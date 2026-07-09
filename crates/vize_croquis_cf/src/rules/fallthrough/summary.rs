use super::FallthroughInfo;

/// Stable counters for fallthrough attribute analysis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FallthroughSummary {
    pub component_count: usize,
    pub components_with_passed_attrs: usize,
    pub components_with_potential_issues: usize,
    pub inherit_attrs_disabled_count: usize,
    pub uses_attrs_count: usize,
    pub binds_attrs_count: usize,
    pub multi_root_count: usize,
    pub multi_root_without_attrs_count: usize,
    pub passed_attr_count: usize,
    pub declared_prop_count: usize,
    pub undeclared_passed_attr_count: usize,
    pub unconsumed_fallthrough_attr_count: usize,
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
        summary.max_passed_attrs = summary.max_passed_attrs.max(info.passed_attrs.len());
        summary.max_root_element_count =
            summary.max_root_element_count.max(info.root_element_count);

        for attr in &info.passed_attrs {
            if !info.declared_props.contains(attr) {
                summary.undeclared_passed_attr_count += 1;
                if !info.uses_attrs && !info.binds_attrs {
                    summary.unconsumed_fallthrough_attr_count += 1;
                }
            }
        }
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
                declared_props: set(&["kind"]),
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
                declared_props: FxHashSet::default(),
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
                declared_props: FxHashSet::default(),
                template_start: 0,
                template_end: 10,
            },
        ];

        let summary = summarize_fallthrough(&infos);

        assert_eq!(summary.component_count, 3);
        assert_eq!(summary.components_with_passed_attrs, 2);
        assert_eq!(summary.components_with_potential_issues, 1);
        assert_eq!(summary.inherit_attrs_disabled_count, 1);
        assert_eq!(summary.uses_attrs_count, 1);
        assert_eq!(summary.binds_attrs_count, 1);
        assert_eq!(summary.multi_root_count, 2);
        assert_eq!(summary.multi_root_without_attrs_count, 1);
        assert_eq!(summary.passed_attr_count, 3);
        assert_eq!(summary.declared_prop_count, 1);
        assert_eq!(summary.undeclared_passed_attr_count, 2);
        assert_eq!(summary.unconsumed_fallthrough_attr_count, 1);
        assert_eq!(summary.max_passed_attrs, 2);
        assert_eq!(summary.max_root_element_count, 3);
    }
}
