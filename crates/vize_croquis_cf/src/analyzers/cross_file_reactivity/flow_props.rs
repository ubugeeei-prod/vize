use super::analyzer::CrossFileReactivityAnalyzer;
use super::prop_helpers::{
    component_usage_targets_child, imported_aliases_for_child, prop_reactivity_loss,
    reactive_source_from_expression,
};
use super::types::{
    CrossFileReactivityIssue, CrossFileReactivityIssueKind, ReactiveValueId, ReactivityFlow,
    ReactivityFlowKind,
};
use crate::diagnostics::DiagnosticSeverity;
use crate::graph::DependencyEdge;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn track_props_flows(&mut self) {
        for node in self.graph.nodes() {
            let parent_file_id = node.file_id;
            let Some(parent_entry) = self.registry.get(parent_file_id) else {
                continue;
            };

            // Check component usages from this file
            for (child_file_id, edge_type) in &node.imports {
                if *edge_type != DependencyEdge::ComponentUsage {
                    continue;
                }

                let Some(child_entry) = self.registry.get(*child_file_id) else {
                    continue;
                };
                let aliases = imported_aliases_for_child(parent_entry, child_entry);

                for usage in &parent_entry.analysis.component_usages {
                    if !component_usage_targets_child(usage.name.as_str(), child_entry, &aliases) {
                        continue;
                    }

                    // Check each prop passed
                    for prop in &usage.props {
                        // Skip if no value
                        let Some(value) = &prop.value else {
                            continue;
                        };

                        // Check if this prop receives a reactive value from the parent.
                        let Some(source) =
                            reactive_source_from_expression(&parent_entry.analysis, value.as_str())
                        else {
                            continue;
                        };

                        let prop_loss =
                            prop_reactivity_loss(&child_entry.analysis, prop.name.as_str());
                        if let Some(loss) = &prop_loss {
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: *child_file_id,
                                kind: CrossFileReactivityIssueKind::ReactivityLostInPropChain {
                                    prop_name: prop.name.clone(),
                                    parent_component: parent_entry
                                        .component_name
                                        .clone()
                                        .unwrap_or_else(|| parent_entry.filename.clone()),
                                },
                                offset: loss.offset,
                                related_file: Some(parent_file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }

                        // Create a props flow
                        let source_id = ReactiveValueId {
                            file_id: parent_file_id,
                            name: source.name.clone(),
                            offset: source.declaration_offset,
                        };
                        let target_id = ReactiveValueId {
                            file_id: *child_file_id,
                            name: prop.name.clone(),
                            offset: prop_loss.as_ref().map_or(0, |loss| loss.offset),
                        };

                        self.flows.push(ReactivityFlow {
                            source: source_id,
                            target: target_id,
                            flow_kind: ReactivityFlowKind::PropsFlow,
                            preserved: prop_loss.is_none(),
                            loss_reason: prop_loss.map(|loss| loss.reason),
                        });
                    }
                }
            }
        }
    }
}
