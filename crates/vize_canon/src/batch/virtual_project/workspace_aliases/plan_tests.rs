use super::*;

fn link(alias: &str, target: &str) -> PackageNodeModulesLink {
    PackageNodeModulesLink {
        virtual_dir: alias.into(),
        real_dir: target.into(),
    }
}

fn topology_candidates(
    topology_id: usize,
    edge_start: usize,
    mut links: Vec<PackageNodeModulesLink>,
    authorities: &FxHashMap<PathBuf, WorkspaceAliasAuthority>,
) -> Vec<WorkspaceAliasCandidate> {
    links.sort_by(|left, right| {
        (&left.virtual_dir, &left.real_dir).cmp(&(&right.virtual_dir, &right.real_dir))
    });
    let edge_ids = links
        .iter()
        .enumerate()
        .map(|(offset, link)| (link.virtual_dir.clone(), edge_start + offset))
        .collect::<FxHashMap<_, _>>();
    links
        .into_iter()
        .enumerate()
        .map(|(offset, link)| WorkspaceAliasCandidate {
            edge_id: edge_start + offset,
            topology_id,
            parent_edge_id: link
                .virtual_dir
                .ancestors()
                .skip(1)
                .find_map(|ancestor| edge_ids.get(ancestor).copied()),
            insertion_authority: authorities
                .get(&link.virtual_dir)
                .copied()
                .unwrap_or_default(),
            raw_alias: link.virtual_dir,
            target: link.real_dir,
        })
        .collect()
}

fn plan_from_topologies(
    topologies: Vec<(
        Vec<PackageNodeModulesLink>,
        FxHashMap<PathBuf, WorkspaceAliasAuthority>,
    )>,
    previous: &WorkspaceAliasPlan,
) -> WorkspaceAliasPlan {
    let mut edge_start = 0;
    let mut candidates = Vec::new();
    for (topology_id, (links, authorities)) in topologies.into_iter().enumerate() {
        let topology = topology_candidates(topology_id, edge_start, links, &authorities);
        edge_start += topology.len();
        candidates.extend(topology);
    }
    WorkspaceAliasPlan::next(candidates, previous)
}

fn plan(
    links: Vec<PackageNodeModulesLink>,
    authorities: FxHashMap<PathBuf, WorkspaceAliasAuthority>,
) -> WorkspaceAliasPlan {
    plan_from_topologies(vec![(links, authorities)], &Default::default())
}

#[test]
fn revision_tracks_exact_plan_bytes() {
    let first = plan(vec![link("alias", "owned")], Default::default());
    let identical = plan_from_topologies(vec![(first.links.clone(), Default::default())], &first);
    assert_eq!(identical, first);
    let changed = plan_from_topologies(
        vec![(vec![link("alias", "next")], Default::default())],
        &first,
    );
    assert_eq!(changed.revision, first.revision + 1);
    assert_ne!(changed.targets, first.targets);
}

#[test]
fn nested_alias_beats_the_direct_collapsed_target() {
    let plan = plan(
        vec![
            link("alias", "owned"),
            link("alias/node_modules/shared", "private/shared"),
            link("owned/node_modules/shared", "direct/shared"),
        ],
        Default::default(),
    );
    assert_eq!(
        plan.targets.get(Path::new("owned/node_modules/shared")),
        Some(&PathBuf::from("private/shared"))
    );
}

fn selected_and_duplicate_parent_fixture(reverse: bool) -> WorkspaceAliasPlan {
    let selected = (
        vec![
            link("root-a", "owned"),
            link("root-a/parent", "private/parent"),
            link("root-a/parent/child", "private/shared"),
        ],
        FxHashMap::from_iter([(
            PathBuf::from("root-a/parent"),
            WorkspaceAliasAuthority::SelectedWorkspaceDependency,
        )]),
    );
    let duplicate = (
        vec![
            link("root-b", "owned"),
            link("root-b/parent", "duplicate/parent"),
            link("root-b/parent/child", "package-root/shared"),
            link("root-b/sibling", "sibling/target"),
        ],
        FxHashMap::from_iter([(
            PathBuf::from("root-b/parent"),
            WorkspaceAliasAuthority::DuplicateTraversalBridge,
        )]),
    );
    let topologies = if reverse {
        vec![duplicate, selected]
    } else {
        vec![selected, duplicate]
    };
    plan_from_topologies(topologies, &Default::default())
}

#[test]
fn duplicate_parent_child_is_rejected_but_its_sibling_survives() {
    let forward = selected_and_duplicate_parent_fixture(false);
    assert_eq!(
        forward.targets.get(Path::new("private/parent/child")),
        Some(&PathBuf::from("private/shared"))
    );
    assert_eq!(
        forward.targets.get(Path::new("owned/sibling")),
        Some(&PathBuf::from("sibling/target"))
    );
    assert!(
        !forward
            .links
            .iter()
            .any(|link| link.real_dir == Path::new("package-root/shared"))
    );
    assert_eq!(forward, selected_and_duplicate_parent_fixture(true));
}

#[test]
fn cross_topology_identical_aliases_deduplicate() {
    let plan = plan_from_topologies(
        vec![
            (
                vec![link("alias", "owned")],
                FxHashMap::from_iter([(
                    PathBuf::from("alias"),
                    WorkspaceAliasAuthority::SelectedWorkspaceDependency,
                )]),
            ),
            (
                vec![link("alias", "owned")],
                FxHashMap::from_iter([(
                    PathBuf::from("alias"),
                    WorkspaceAliasAuthority::DuplicateTraversalBridge,
                )]),
            ),
        ],
        &Default::default(),
    );
    assert_eq!(plan.links, vec![link("alias", "owned")]);
}

#[test]
fn equal_authority_parent_conflict_stays_fail_closed() {
    let plan = plan(
        vec![
            link("root-a", "owned"),
            link("root-b", "owned"),
            link("root-a/parent", "private/a"),
            link("root-b/parent", "private/b"),
            link("root-a/parent/child", "private/a-child"),
            link("root-b/parent/child", "private/b-child"),
        ],
        Default::default(),
    );
    assert!(!plan.targets.contains_key(Path::new("owned/parent")));
    assert_eq!(
        plan.links,
        vec![
            link("owned/parent", "private/a"),
            link("owned/parent", "private/b"),
            link("root-a", "owned"),
            link("root-b", "owned"),
        ]
    );
}

#[test]
fn target_chains_resolve_and_cycles_stay_visible() {
    let chain = plan(
        vec![
            link("alias", "alias/child"),
            link("alias/child", "owned/child"),
        ],
        Default::default(),
    );
    assert_eq!(
        chain.targets.get(Path::new("alias")),
        Some(&PathBuf::from("owned/child"))
    );
    let cycle = plan(
        vec![link("alias-a", "alias-b"), link("alias-b", "alias-a")],
        Default::default(),
    );
    assert_eq!(
        cycle.targets.get(Path::new("alias-a")),
        Some(&PathBuf::from("alias-b"))
    );
    assert_eq!(
        cycle.targets.get(Path::new("alias-b")),
        Some(&PathBuf::from("alias-a"))
    );
}

#[test]
fn receipt_paths_with_parent_escapes_stay_rejected() {
    assert!(!is_normalized_descendant(
        Path::new("/workspace/virtual/../escape"),
        Path::new("/workspace/virtual")
    ));
}
