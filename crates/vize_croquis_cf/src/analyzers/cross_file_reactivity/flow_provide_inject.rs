use super::analyzer::CrossFileReactivityAnalyzer;
use super::provide_helpers::{provide_key_display, provide_key_identity};
use super::types::{
    CrossFileReactivityIssue, CrossFileReactivityIssueKind, ProvideDefinition, ReactiveValueId,
    ReactivityFlow, ReactivityFlowKind, ReactivityLossReason,
};
use crate::diagnostics::DiagnosticSeverity;
use crate::graph::DependencyEdge;
use crate::registry::FileId;
use vize_carton::{CompactString, FxHashSet};

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn track_provide_inject_flows(&mut self) {
        for entry in self.registry.vue_components() {
            let consumer_file_id = entry.id;
            let analysis = &entry.analysis;

            for inject in analysis.provide_inject.injects() {
                let key_str = provide_key_display(&inject.key);
                let key_identity = provide_key_identity(&inject.key);

                // Find providers in every ancestor branch. A component can be reused
                // under multiple parents, so a single inject can have multiple
                // runtime provider contexts.
                for provider in self.find_nearest_providers(consumer_file_id, key_identity.as_str())
                {
                    // Check if inject result is destructured
                    use vize_croquis::provide::InjectPattern;
                    match &inject.pattern {
                        InjectPattern::ObjectDestructure(props) => {
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: props.clone(),
                                },
                                offset: inject.start,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::ArrayDestructure(_) => {
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: vec![CompactString::new(
                                        "(array destructure)",
                                    )],
                                },
                                offset: inject.start,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::IndirectDestructure { props, offset, .. } => {
                            // Indirect destructuring also loses reactivity
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: props.clone(),
                                },
                                offset: *offset,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::Simple => {
                            // OK - inject is assigned to a variable
                        }
                    }

                    // Check if provider provides non-reactive value
                    if !provider.is_reactive {
                        self.issues.push(CrossFileReactivityIssue {
                            file_id: provider.file_id,
                            kind: CrossFileReactivityIssueKind::NonReactiveProvide {
                                key: provider.key.clone(),
                            },
                            offset: provider.offset,
                            related_file: Some(consumer_file_id),
                            severity: DiagnosticSeverity::Warning,
                        });
                    }

                    // Create a flow record
                    let source_id = ReactiveValueId {
                        file_id: provider.file_id,
                        name: provider.value_name.clone(),
                        offset: provider.offset,
                    };
                    let target_id = ReactiveValueId {
                        file_id: consumer_file_id,
                        name: inject.local_name.clone(),
                        offset: inject.start,
                    };

                    let (preserved, loss_reason) = match &inject.pattern {
                        InjectPattern::Simple => (true, None),
                        InjectPattern::ObjectDestructure(_props) => {
                            (false, Some(ReactivityLossReason::InjectDestructure))
                        }
                        InjectPattern::ArrayDestructure(_) => (
                            false,
                            Some(ReactivityLossReason::Destructured { props: vec![] }),
                        ),
                        InjectPattern::IndirectDestructure { .. } => {
                            (false, Some(ReactivityLossReason::InjectDestructure))
                        }
                    };

                    self.flows.push(ReactivityFlow {
                        source: source_id,
                        target: target_id,
                        flow_kind: ReactivityFlowKind::ProvideInject,
                        preserved,
                        loss_reason,
                    });
                }
            }
        }
    }

    pub(super) fn find_nearest_providers(
        &self,
        consumer_file_id: FileId,
        key_identity: &str,
    ) -> Vec<ProvideDefinition> {
        let mut providers = Vec::new();
        let mut seen_providers = FxHashSet::default();
        // Parent-pointer BFS frames: each frame records the visited file and the
        // index of the frame it was reached from. The visited path (used only for
        // cycle detection) is recovered by walking `parent` pointers, which avoids
        // cloning an O(depth) `Vec<FileId>` for every queued node.
        let mut frames = vec![AncestorFrame {
            current: consumer_file_id,
            parent: None,
        }];
        let mut cursor = 0;

        while cursor < frames.len() {
            let frame_index = cursor;
            let current = frames[frame_index].current;
            cursor += 1;

            if current != consumer_file_id
                && let Some(provides) = self.provides.get(&current)
                && let Some(provider) = provides
                    .iter()
                    .rev()
                    .find(|provider| provider.key_identity.as_str() == key_identity)
            {
                if seen_providers.insert((provider.file_id, provider.offset)) {
                    providers.push(provider.clone());
                }
                continue;
            }

            let mut parents: Vec<_> = self
                .graph
                .dependents(current)
                .filter(|(parent_id, edge_type)| {
                    *edge_type == DependencyEdge::ComponentUsage
                        && !frame_contains(&frames, frame_index, *parent_id)
                })
                .collect();
            parents.sort_by_key(|(parent_id, _)| parent_id.as_u32());

            for (parent_id, _) in parents {
                frames.push(AncestorFrame {
                    current: parent_id,
                    parent: Some(frame_index),
                });
            }
        }

        providers.sort_by_key(|provider| (provider.file_id.as_u32(), provider.offset));
        providers
    }
}

#[derive(Debug, Clone, Copy)]
struct AncestorFrame {
    current: FileId,
    parent: Option<usize>,
}

/// Returns true if `needle` appears on the visited path ending at `index`,
/// walking `parent` pointers to the root. Mirrors `path.contains(..)` over the
/// path that the original `(FileId, Vec<FileId>)` queue accumulated.
fn frame_contains(frames: &[AncestorFrame], mut index: usize, needle: FileId) -> bool {
    loop {
        let frame = frames[index];
        if frame.current == needle {
            return true;
        }
        let Some(parent) = frame.parent else {
            return false;
        };
        index = parent;
    }
}
