//! Move nested dependency links through an owned workspace alias target.

use std::cmp::Reverse;
use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::super::package_node_modules::PackageNodeModulesLink;
use super::super::package_shadow::WorkspaceAliasAuthority;
use super::WorkspaceAliasCandidate;

pub(super) fn descendants(
    links: &mut [PackageNodeModulesLink],
    aliases: &FxHashMap<PathBuf, PathBuf>,
) {
    for link in links {
        if let Some(rebased) = descendant_path(&link.virtual_dir, aliases) {
            link.virtual_dir = rebased;
        }
    }
}

#[derive(Debug)]
struct StagedAlias {
    candidate: WorkspaceAliasCandidate,
    virtual_dir: PathBuf,
    rebase_depth: usize,
}

#[derive(Debug)]
struct TargetGroup {
    target: PathBuf,
    priority: (usize, WorkspaceAliasAuthority),
    candidates: Vec<WorkspaceAliasCandidate>,
}

/// Select raw parents before rebasing their descendants.
///
/// A rejected duplicate traversal must not contribute children to the
/// selected parent's namespace. Processing virtual path depths root-first
/// gives every child one immutable set of selected parent links. Equal
/// rebase-depth and authority conflicts remain present so plan validation
/// rejects them instead of selecting by iteration.
pub(super) fn plan_aliases(mut raw: Vec<WorkspaceAliasCandidate>) -> Vec<PackageNodeModulesLink> {
    raw.sort_by(|left, right| {
        left.raw_alias
            .components()
            .count()
            .cmp(&right.raw_alias.components().count())
            .then_with(|| {
                (
                    &left.raw_alias,
                    &left.target,
                    left.topology_id,
                    left.edge_id,
                )
                    .cmp(&(
                        &right.raw_alias,
                        &right.target,
                        right.topology_id,
                        right.edge_id,
                    ))
            })
    });

    let mut links = Vec::new();
    let mut selected_parents = FxHashMap::<PathBuf, PathBuf>::default();
    let mut rejected = FxHashSet::<usize>::default();
    let mut raw = raw.into_iter().peekable();
    while let Some(raw_depth) = raw
        .peek()
        .map(|candidate| candidate.raw_alias.components().count())
    {
        let mut staged = Vec::new();
        while let Some(candidate) =
            raw.next_if(|candidate| candidate.raw_alias.components().count() == raw_depth)
        {
            if candidate
                .parent_edge_id
                .is_some_and(|parent| rejected.contains(&parent))
            {
                rejected.insert(candidate.edge_id);
                continue;
            }
            let (virtual_dir, rebase_depth) =
                descendant_path_with_depth(&candidate.raw_alias, &selected_parents)
                    .unwrap_or_else(|| (candidate.raw_alias.clone(), 0));
            staged.push(StagedAlias {
                candidate,
                virtual_dir,
                rebase_depth,
            });
        }
        staged.sort_by(|left, right| {
            (
                &left.virtual_dir,
                &left.candidate.target,
                Reverse(left.rebase_depth),
                Reverse(left.candidate.insertion_authority),
                left.candidate.topology_id,
                left.candidate.edge_id,
            )
                .cmp(&(
                    &right.virtual_dir,
                    &right.candidate.target,
                    Reverse(right.rebase_depth),
                    Reverse(right.candidate.insertion_authority),
                    right.candidate.topology_id,
                    right.candidate.edge_id,
                ))
        });

        let mut staged = staged.into_iter().peekable();
        let mut path_candidates = Vec::new();
        while let Some(staged_candidate) = staged.next() {
            let selected_path = staged_candidate.virtual_dir.clone();
            path_candidates.push(staged_candidate);
            while let Some(staged_candidate) =
                staged.next_if(|candidate| candidate.virtual_dir == selected_path)
            {
                path_candidates.push(staged_candidate);
            }

            let mut grouped = path_candidates.drain(..).peekable();
            let mut target_groups = Vec::new();
            while let Some(staged_candidate) = grouped.next() {
                let target = staged_candidate.candidate.target.clone();
                let mut priority = (
                    staged_candidate.rebase_depth,
                    staged_candidate.candidate.insertion_authority,
                );
                let mut candidates = vec![staged_candidate.candidate];
                while let Some(staged_candidate) =
                    grouped.next_if(|candidate| candidate.candidate.target == target)
                {
                    priority = priority.max((
                        staged_candidate.rebase_depth,
                        staged_candidate.candidate.insertion_authority,
                    ));
                    candidates.push(staged_candidate.candidate);
                }
                target_groups.push(TargetGroup {
                    target,
                    priority,
                    candidates,
                });
            }
            let selected_priority = target_groups
                .iter()
                .map(|group| group.priority)
                .max()
                .unwrap_or_default();
            let conflict = target_groups
                .iter()
                .filter(|group| group.priority == selected_priority)
                .nth(1)
                .is_some();
            for group in target_groups {
                let selected = group.priority == selected_priority;
                if selected {
                    links.push(PackageNodeModulesLink {
                        virtual_dir: selected_path.clone(),
                        real_dir: group.target.clone(),
                    });
                }
                for candidate in group.candidates {
                    if selected && !conflict {
                        selected_parents.insert(candidate.raw_alias, group.target.clone());
                    } else {
                        rejected.insert(candidate.edge_id);
                    }
                }
            }
        }
    }
    for link in &mut links {
        if let Some(target) = target_path(&link.real_dir, &selected_parents) {
            link.real_dir = target;
        }
    }
    links
}

pub(super) fn descendants_with_owned_priority(
    links: &mut Vec<PackageNodeModulesLink>,
    aliases: &FxHashMap<PathBuf, PathBuf>,
) {
    descendants(links, aliases);
    links.retain(|link| {
        aliases
            .get(&link.virtual_dir)
            .is_none_or(|target| target == &link.real_dir)
    });
}

fn descendant_path(original: &Path, aliases: &FxHashMap<PathBuf, PathBuf>) -> Option<PathBuf> {
    descendant_path_with_depth(original, aliases).map(|(path, _)| path)
}

fn descendant_path_with_depth(
    original: &Path,
    aliases: &FxHashMap<PathBuf, PathBuf>,
) -> Option<(PathBuf, usize)> {
    let mut current = original.to_path_buf();
    let mut visited = FxHashSet::default();
    let mut depth = 0;
    loop {
        let Some((alias, target)) = current
            .ancestors()
            .skip(1)
            .find_map(|candidate| aliases.get_key_value(candidate))
        else {
            return Some((current, depth));
        };
        if !visited.insert(alias.clone()) {
            return None;
        }
        let suffix = current.strip_prefix(alias).ok()?;
        let next = target.join(suffix);
        if next == current {
            return None;
        }
        current = next;
        depth += 1;
    }
}

fn target_path(original: &Path, aliases: &FxHashMap<PathBuf, PathBuf>) -> Option<PathBuf> {
    let mut current = original.to_path_buf();
    let mut visited = FxHashSet::default();
    loop {
        let Some((alias, target)) = current
            .ancestors()
            .find_map(|candidate| aliases.get_key_value(candidate))
        else {
            return Some(current);
        };
        if !visited.insert(alias.clone()) {
            return None;
        }
        let suffix = current.strip_prefix(alias).ok()?;
        let next = target.join(suffix);
        if next == current {
            return None;
        }
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(virtual_dir: &str, real_dir: &str) -> PackageNodeModulesLink {
        PackageNodeModulesLink {
            virtual_dir: virtual_dir.into(),
            real_dir: real_dir.into(),
        }
    }

    #[test]
    fn nested_dependency_links_collapse_at_the_owned_alias_target() {
        let aliases = FxHashMap::from_iter([("alias".into(), "owned".into())]);
        let mut links = vec![
            link("alias", "owned"),
            link("alias/node_modules/shared", "real/shared"),
            link("owned/node_modules/shared", "real/shared"),
            link("sibling/node_modules/shared", "real/shared"),
        ];
        descendants(&mut links, &aliases);
        links.sort_by(|left, right| {
            (&left.virtual_dir, &left.real_dir).cmp(&(&right.virtual_dir, &right.real_dir))
        });
        links.dedup();
        assert_eq!(
            links,
            vec![
                link("alias", "owned"),
                link("owned/node_modules/shared", "real/shared"),
                link("sibling/node_modules/shared", "real/shared"),
            ]
        );
    }

    #[test]
    fn alias_cycles_and_owned_alias_links_stay_fail_closed() {
        let aliases = FxHashMap::from_iter([
            ("alias-a".into(), "alias-b".into()),
            ("alias-b".into(), "alias-a".into()),
        ]);
        let mut links = vec![
            link("alias-a", "alias-b"),
            link("alias-a/node_modules/shared", "real/shared"),
        ];
        let original = links.clone();
        descendants(&mut links, &aliases);
        assert_eq!(links, original);
    }

    #[test]
    fn nested_owned_aliases_move_below_the_parent_target() {
        let aliases = FxHashMap::from_iter([
            ("alias".into(), "owned".into()),
            ("alias/node_modules/shared".into(), "shared-owned".into()),
        ]);
        let mut links = vec![
            link("alias", "owned"),
            link("alias/node_modules/shared", "shared-owned"),
        ];
        descendants(&mut links, &aliases);
        assert_eq!(
            links,
            vec![
                link("alias", "owned"),
                link("owned/node_modules/shared", "shared-owned"),
            ]
        );
    }

    #[test]
    fn owned_nested_alias_wins_the_rebased_dependency_path() {
        let aliases = FxHashMap::from_iter([
            ("alias".into(), "owned".into()),
            ("owned/node_modules/shared".into(), "shared-owned".into()),
        ]);
        let mut links = vec![
            link("alias", "owned"),
            link("owned/node_modules/shared", "shared-owned"),
            link("alias/node_modules/shared", "external/shared"),
        ];
        descendants_with_owned_priority(&mut links, &aliases);
        assert_eq!(
            links,
            vec![
                link("alias", "owned"),
                link("owned/node_modules/shared", "shared-owned"),
            ]
        );
    }
}
