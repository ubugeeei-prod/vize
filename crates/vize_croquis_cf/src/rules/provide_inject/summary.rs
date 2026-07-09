use super::types::{ProvideInjectTree, ProvideNode};

/// Stable counters for a provide/inject tree.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProvideInjectTreeSummary {
    pub root_count: usize,
    pub node_count: usize,
    pub provider_component_count: usize,
    pub injector_component_count: usize,
    pub provide_count: usize,
    pub inject_count: usize,
    pub matched_inject_count: usize,
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

        for root in &self.roots {
            summarize_node(root, 1, &mut summary);
        }

        summary
    }
}

fn summarize_node(node: &ProvideNode, depth: usize, summary: &mut ProvideInjectTreeSummary) {
    summary.node_count += 1;
    summary.max_depth = summary.max_depth.max(depth);
    summary.max_child_fanout = summary.max_child_fanout.max(node.children.len());

    if !node.provides.is_empty() {
        summary.provider_component_count += 1;
    }
    summary.provide_count += node.provides.len();

    for provide in &node.provides {
        summary.max_provider_consumer_count = summary
            .max_provider_consumer_count
            .max(provide.consumer_count);
    }

    if !node.injects.is_empty() {
        summary.injector_component_count += 1;
    }
    summary.inject_count += node.injects.len();

    for inject in &node.injects {
        if inject.provider.is_some() {
            summary.matched_inject_count += 1;
        } else {
            summary.unmatched_inject_count += 1;
        }
    }

    for child in &node.children {
        summarize_node(child, depth + 1, summary);
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
        assert_eq!(summary.provider_component_count, 1);
        assert_eq!(summary.injector_component_count, 2);
        assert_eq!(summary.provide_count, 1);
        assert_eq!(summary.inject_count, 2);
        assert_eq!(summary.matched_inject_count, 1);
        assert_eq!(summary.unmatched_inject_count, 1);
        assert_eq!(summary.max_depth, 3);
        assert_eq!(summary.max_child_fanout, 2);
        assert_eq!(summary.max_provider_consumer_count, 2);
    }
}
