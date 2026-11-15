//! Compact topology-local inputs for one workspace-alias transaction.

use vize_carton::FxHashMap;

use super::{VirtualProject, WorkspaceAliasCandidate};

impl VirtualProject {
    pub(super) fn raw_workspace_alias_candidates(&self) -> Vec<WorkspaceAliasCandidate> {
        let mut topologies = self.package_shadow_artifacts.iter().collect::<Vec<_>>();
        topologies.sort_by_key(|(left, _)| *left);
        let mut candidates = Vec::new();
        for (topology_id, (_, topology)) in topologies.into_iter().enumerate() {
            let mut aliases = topology
                .aliases
                .iter()
                .map(|(alias, target)| (alias.clone(), target.clone()))
                .collect::<Vec<_>>();
            aliases.sort();
            let edge_start = candidates.len();
            let edge_ids = aliases
                .iter()
                .enumerate()
                .map(|(offset, (alias, _))| (alias.clone(), edge_start + offset))
                .collect::<FxHashMap<_, _>>();
            for (offset, (raw_alias, target)) in aliases.into_iter().enumerate() {
                let parent_edge_id = raw_alias
                    .ancestors()
                    .skip(1)
                    .find_map(|ancestor| edge_ids.get(ancestor).copied());
                candidates.push(WorkspaceAliasCandidate {
                    edge_id: edge_start + offset,
                    topology_id,
                    parent_edge_id,
                    insertion_authority: topology.alias_authority(&raw_alias),
                    raw_alias,
                    target,
                });
            }
        }
        candidates
    }
}
