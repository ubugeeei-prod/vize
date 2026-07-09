use super::{FallthroughInfo, FallthroughUsageAttrKind, FallthroughUsageFact};
use crate::registry::FileId;
use serde::Serialize;
use vize_carton::{FxHashMap, FxHashSet};

/// Stable per-component aggregate for fallthrough attribute analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallthroughComponentFact {
    pub file_id: FileId,
    pub usage_count: usize,
    pub parent_count: usize,
    pub spread_usage_count: usize,
    pub passed_attr_count: usize,
    pub usage_attr_count: usize,
    pub prop_attr_count: usize,
    pub listener_attr_count: usize,
    pub dynamic_attr_count: usize,
    pub declared_prop_attr_count: usize,
    pub declared_event_attr_count: usize,
    pub fallthrough_attr_count: usize,
    pub consumed_fallthrough_attr_count: usize,
    pub unconsumed_fallthrough_attr_count: usize,
    pub safe_standard_fallthrough_attr_count: usize,
    pub risky_unconsumed_fallthrough_attr_count: usize,
    pub declared_prop_count: usize,
    pub declared_event_count: usize,
    pub root_element_count: usize,
    pub inherit_attrs_disabled: bool,
    pub uses_attrs: bool,
    pub binds_attrs: bool,
    pub has_potential_issues: bool,
}

impl Default for FallthroughComponentFact {
    fn default() -> Self {
        Self {
            file_id: FileId::INVALID,
            usage_count: 0,
            parent_count: 0,
            spread_usage_count: 0,
            passed_attr_count: 0,
            usage_attr_count: 0,
            prop_attr_count: 0,
            listener_attr_count: 0,
            dynamic_attr_count: 0,
            declared_prop_attr_count: 0,
            declared_event_attr_count: 0,
            fallthrough_attr_count: 0,
            consumed_fallthrough_attr_count: 0,
            unconsumed_fallthrough_attr_count: 0,
            safe_standard_fallthrough_attr_count: 0,
            risky_unconsumed_fallthrough_attr_count: 0,
            declared_prop_count: 0,
            declared_event_count: 0,
            root_element_count: 0,
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            has_potential_issues: false,
        }
    }
}

pub fn collect_fallthrough_component_facts(
    infos: &[FallthroughInfo],
    usage_facts: &[FallthroughUsageFact],
) -> Vec<FallthroughComponentFact> {
    let mut facts = infos
        .iter()
        .map(FallthroughComponentFact::from_info)
        .collect::<Vec<_>>();
    let indexes = facts
        .iter()
        .enumerate()
        .map(|(index, fact)| (fact.file_id, index))
        .collect::<FxHashMap<_, _>>();
    let mut parent_sets = vec![FxHashSet::default(); facts.len()];

    for usage in usage_facts {
        let Some(&index) = indexes.get(&usage.child_file_id) else {
            continue;
        };
        let fact = &mut facts[index];
        fact.usage_count += 1;
        fact.spread_usage_count += usize::from(usage.has_spread_attrs);
        parent_sets[index].insert(usage.parent_file_id);

        for attr in &usage.attrs {
            fact.usage_attr_count += 1;
            fact.dynamic_attr_count += usize::from(attr.dynamic);
            fact.declared_prop_attr_count += usize::from(attr.declared_prop);
            fact.declared_event_attr_count += usize::from(attr.declared_event);
            match attr.kind {
                FallthroughUsageAttrKind::Prop => fact.prop_attr_count += 1,
                FallthroughUsageAttrKind::Listener => fact.listener_attr_count += 1,
            }
        }
    }

    for (fact, parents) in facts.iter_mut().zip(parent_sets) {
        fact.parent_count = parents.len();
    }
    facts.sort_unstable_by_key(|fact| fact.file_id.as_u32());
    facts
}

impl FallthroughComponentFact {
    fn from_info(info: &FallthroughInfo) -> Self {
        Self {
            file_id: info.file_id,
            passed_attr_count: info.passed_attrs.len(),
            fallthrough_attr_count: info.fallthrough_attr_count(),
            consumed_fallthrough_attr_count: info.consumed_fallthrough_attr_count(),
            unconsumed_fallthrough_attr_count: info.unconsumed_fallthrough_attr_count(),
            safe_standard_fallthrough_attr_count: info.safe_standard_fallthrough_attr_count(),
            risky_unconsumed_fallthrough_attr_count: info.risky_unconsumed_fallthrough_attr_count(),
            declared_prop_count: info.declared_props.len(),
            declared_event_count: info.declared_events.len(),
            root_element_count: info.root_element_count,
            inherit_attrs_disabled: info.inherit_attrs_disabled,
            uses_attrs: info.uses_attrs,
            binds_attrs: info.binds_attrs,
            has_potential_issues: info.has_potential_issues(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fallthrough::FallthroughUsageAttrFact;
    use vize_carton::CompactString;

    fn set(names: &[&str]) -> FxHashSet<CompactString> {
        names.iter().map(|name| CompactString::new(*name)).collect()
    }

    fn attr(
        name: &str,
        kind: FallthroughUsageAttrKind,
        dynamic: bool,
        declared_prop: bool,
        declared_event: bool,
        standard_html_attr: bool,
    ) -> FallthroughUsageAttrFact {
        FallthroughUsageAttrFact {
            name: CompactString::new(name),
            kind,
            source_start: 0,
            source_end: 0,
            dynamic,
            declared_prop,
            declared_event,
            standard_html_attr,
            fallthrough: !declared_prop && !declared_event,
        }
    }

    #[test]
    fn component_facts_preserve_usage_and_component_counts() {
        let child = FileId::new(2);
        let infos = vec![FallthroughInfo {
            file_id: child,
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 2,
            passed_attrs: set(&["kind", "trackingId", "onClose"]),
            fallthrough_attrs: set(&["trackingId"]),
            declared_props: set(&["kind"]),
            declared_events: set(&["close"]),
            template_start: 0,
            template_end: 10,
        }];
        let usage_facts = vec![
            FallthroughUsageFact {
                parent_file_id: FileId::new(1),
                child_file_id: child,
                component_name: CompactString::new("Child"),
                usage_start: 0,
                usage_end: 0,
                has_spread_attrs: true,
                attrs: vec![
                    attr(
                        "kind",
                        FallthroughUsageAttrKind::Prop,
                        false,
                        true,
                        false,
                        false,
                    ),
                    attr(
                        "trackingId",
                        FallthroughUsageAttrKind::Prop,
                        true,
                        false,
                        false,
                        false,
                    ),
                ],
            },
            FallthroughUsageFact {
                parent_file_id: FileId::new(3),
                child_file_id: child,
                component_name: CompactString::new("Child"),
                usage_start: 0,
                usage_end: 0,
                has_spread_attrs: false,
                attrs: vec![attr(
                    "onClose",
                    FallthroughUsageAttrKind::Listener,
                    true,
                    false,
                    true,
                    true,
                )],
            },
        ];

        let facts = collect_fallthrough_component_facts(&infos, &usage_facts);
        let fact = &facts[0];

        assert_eq!(fact.usage_count, 2);
        assert_eq!(fact.parent_count, 2);
        assert_eq!(fact.spread_usage_count, 1);
        assert_eq!(fact.passed_attr_count, 3);
        assert_eq!(fact.usage_attr_count, 3);
        assert_eq!(fact.prop_attr_count, 2);
        assert_eq!(fact.listener_attr_count, 1);
        assert_eq!(fact.dynamic_attr_count, 2);
        assert_eq!(fact.declared_prop_attr_count, 1);
        assert_eq!(fact.declared_event_attr_count, 1);
        assert_eq!(fact.declared_event_count, 1);
        assert_eq!(fact.fallthrough_attr_count, 1);
        assert_eq!(fact.safe_standard_fallthrough_attr_count, 0);
        assert_eq!(fact.risky_unconsumed_fallthrough_attr_count, 1);
        assert!(fact.has_potential_issues);

        let json = serde_json::to_value(fact).unwrap();
        assert_eq!(json["spreadUsageCount"], 1);
        assert_eq!(json["listenerAttrCount"], 1);
        assert_eq!(json["declaredEventAttrCount"], 1);
        assert_eq!(json["declaredEventCount"], 1);
    }

    #[test]
    fn component_facts_keep_zero_usage_components() {
        let infos = vec![FallthroughInfo {
            file_id: FileId::new(5),
            inherit_attrs_disabled: true,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 1,
            passed_attrs: FxHashSet::default(),
            fallthrough_attrs: FxHashSet::default(),
            declared_props: set(&["kind"]),
            declared_events: FxHashSet::default(),
            template_start: 0,
            template_end: 10,
        }];

        let facts = collect_fallthrough_component_facts(&infos, &[]);

        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].usage_count, 0);
        assert_eq!(facts[0].declared_prop_count, 1);
        assert!(facts[0].inherit_attrs_disabled);
        assert!(facts[0].has_potential_issues);
    }
}
