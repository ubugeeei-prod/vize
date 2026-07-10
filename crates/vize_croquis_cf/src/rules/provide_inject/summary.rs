use super::types::{ProvideInjectTree, ProvideNode};
use crate::registry::FileId;
use serde::Serialize;
use vize_carton::{FxHashMap, FxHashSet};

/// Stable counters for a provide/inject tree.
///
/// Component and call counters are unique by file and source offset so a
/// component reused in multiple render branches is not counted repeatedly.
/// Depth and fan-out still describe the rendered branch occurrences.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvideInjectTreeSummary {
    pub root_count: usize,
    pub node_count: usize,
    pub leaf_component_count: usize,
    pub pass_through_component_count: usize,
    pub provider_component_count: usize,
    pub injector_component_count: usize,
    pub provide_count: usize,
    pub inject_count: usize,
    pub defaulted_inject_count: usize,
    /// Inject calls whose every rendered branch has a provider.
    pub matched_inject_count: usize,
    /// Inject calls missing a provider on at least one rendered branch.
    pub unmatched_inject_count: usize,
    pub max_depth: usize,
    pub max_child_fanout: usize,
    pub max_provider_consumer_count: usize,
}

impl ProvideInjectTree {
    /// Summarize the tree for reports and cross-file complexity scoring.
    pub fn summary(&self) -> ProvideInjectTreeSummary {
        let mut summary = ProvideInjectTreeSummary {
            root_count: self.roots.len(),
            ..ProvideInjectTreeSummary::default()
        };
        let mut state = SummaryState::default();

        for root in &self.roots {
            summarize_node(root, 1, &mut summary, &mut state);
        }

        summary.node_count = state.components.len();
        for component in state.components.values() {
            if !component.has_children {
                summary.leaf_component_count += 1;
            } else if !component.has_provides && !component.has_injects {
                summary.pass_through_component_count += 1;
            }
            if component.has_provides {
                summary.provider_component_count += 1;
            }
            if component.has_injects {
                summary.injector_component_count += 1;
            }
        }

        summary.provide_count = state.provides.len();
        summary.inject_count = state.injects.len();
        for inject in state.injects.values() {
            if inject.has_default {
                summary.defaulted_inject_count += 1;
            }
            if inject.has_unmatched_branch {
                summary.unmatched_inject_count += 1;
            } else if inject.has_matched_branch {
                summary.matched_inject_count += 1;
            }
        }

        summary
    }
}

#[derive(Debug, Default)]
struct SummaryState {
    components: FxHashMap<FileId, ComponentState>,
    provides: FxHashSet<(FileId, u32)>,
    injects: FxHashMap<(FileId, u32), InjectState>,
}

#[derive(Debug, Default)]
struct ComponentState {
    has_children: bool,
    has_provides: bool,
    has_injects: bool,
}

#[derive(Debug, Default)]
struct InjectState {
    has_default: bool,
    has_matched_branch: bool,
    has_unmatched_branch: bool,
}

fn summarize_node(
    node: &ProvideNode,
    depth: usize,
    summary: &mut ProvideInjectTreeSummary,
    state: &mut SummaryState,
) {
    summary.max_depth = summary.max_depth.max(depth);
    summary.max_child_fanout = summary.max_child_fanout.max(node.children.len());

    let component = state.components.entry(node.file_id).or_default();
    component.has_children |= !node.children.is_empty();
    component.has_provides |= !node.provides.is_empty();
    component.has_injects |= !node.injects.is_empty();

    for provide in &node.provides {
        state.provides.insert((node.file_id, provide.offset));
        summary.max_provider_consumer_count = summary
            .max_provider_consumer_count
            .max(provide.consumer_count);
    }

    for inject in &node.injects {
        let inject_state = state
            .injects
            .entry((node.file_id, inject.offset))
            .or_default();
        inject_state.has_default |= inject.has_default;
        if inject.provider.is_some() {
            inject_state.has_matched_branch = true;
        } else {
            inject_state.has_unmatched_branch = true;
        }
    }

    for child in &node.children {
        summarize_node(child, depth + 1, summary, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::FileId;
    use vize_carton::CompactString;

    fn file_id(id: u32) -> FileId {
        FileId::new(id)
    }

    #[test]
    fn summary_counts_tree_depth_fanout_and_match_state() {
        let tree = ProvideInjectTree {
            roots: vec![ProvideNode {
                file_id: file_id(1),
                component_name: Some(CompactString::new("Root")),
                provides: vec![super::super::types::ProvideInfo {
                    key: CompactString::new("theme"),
                    value_type: None,
                    offset: 10,
                    consumer_count: 2,
                }],
                injects: Vec::new(),
                children: vec![
                    ProvideNode {
                        file_id: file_id(2),
                        component_name: Some(CompactString::new("Panel")),
                        provides: Vec::new(),
                        injects: vec![super::super::types::InjectInfo {
                            key: CompactString::new("theme"),
                            has_default: false,
                            provider: Some(file_id(1)),
                            offset: 20,
                        }],
                        children: vec![ProvideNode {
                            file_id: file_id(3),
                            component_name: Some(CompactString::new("Leaf")),
                            provides: Vec::new(),
                            injects: vec![super::super::types::InjectInfo {
                                key: CompactString::new("locale"),
                                has_default: true,
                                provider: None,
                                offset: 30,
                            }],
                            children: Vec::new(),
                        }],
                    },
                    ProvideNode {
                        file_id: file_id(4),
                        component_name: Some(CompactString::new("Sidebar")),
                        provides: Vec::new(),
                        injects: Vec::new(),
                        children: Vec::new(),
                    },
                ],
            }],
        };

        let summary = tree.summary();

        assert_eq!(summary.root_count, 1);
        assert_eq!(summary.node_count, 4);
        assert_eq!(summary.leaf_component_count, 2);
        assert_eq!(summary.pass_through_component_count, 0);
        assert_eq!(summary.provider_component_count, 1);
        assert_eq!(summary.injector_component_count, 2);
        assert_eq!(summary.provide_count, 1);
        assert_eq!(summary.inject_count, 2);
        assert_eq!(summary.defaulted_inject_count, 1);
        assert_eq!(summary.matched_inject_count, 1);
        assert_eq!(summary.unmatched_inject_count, 1);
        assert_eq!(summary.max_depth, 3);
        assert_eq!(summary.max_child_fanout, 2);
        assert_eq!(summary.max_provider_consumer_count, 2);

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains(r#""defaultedInjectCount":1"#));
    }

    #[test]
    fn summary_classifies_a_partially_matched_call_as_unmatched() {
        let shared_inject = |provider| ProvideNode {
            file_id: file_id(3),
            component_name: Some(CompactString::new("Shared")),
            provides: Vec::new(),
            injects: vec![super::super::types::InjectInfo {
                key: CompactString::new("theme"),
                has_default: false,
                provider,
                offset: 20,
            }],
            children: Vec::new(),
        };
        let root = |id, child| ProvideNode {
            file_id: file_id(id),
            component_name: Some(CompactString::new("Root")),
            provides: Vec::new(),
            injects: Vec::new(),
            children: vec![child],
        };
        let tree = ProvideInjectTree {
            roots: vec![
                root(1, shared_inject(Some(file_id(1)))),
                root(2, shared_inject(None)),
            ],
        };

        let summary = tree.summary();
        assert_eq!(summary.inject_count, 1);
        assert_eq!(summary.matched_inject_count, 0);
        assert_eq!(summary.unmatched_inject_count, 1);
    }
}
