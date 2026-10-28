use std::path::{Path, PathBuf};

use super::{
    CanonicalPathCache, ImportFileOptions, PathAliasResolver, resolve_local,
    scan_route_reachability,
};
use vize_canon::batch::{ReachabilityOutcome, ReachabilitySources};

fn write(root: &Path, name: &str, content: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
    path
}

fn options() -> ImportFileOptions {
    ImportFileOptions::from(false)
}

fn route(root: &Path, entry: PathBuf) -> vize_canon::PackageRoute {
    vize_canon::PackageRoute {
        source_paths: vec![entry],
        dependency_paths: Vec::new(),
        source_targets: Vec::new(),
        package_root: root.to_path_buf(),
        package_link_root: root.to_path_buf(),
        manifest_path: root.join("package.json"),
        package_name: Some("cold-registration-control".into()),
        workspace_source: false,
        nested_routes: Vec::new(),
    }
}

#[test]
fn untracked_probes_keep_typescript_precedence_and_omit_candidate_histories() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let importer = write(&root, "entry.ts", "export {};\n");
    let model = write(&root, "model.ts", "export const n: number = 1;\n");
    write(&root, "model.js", "export const n = 'javascript';\n");
    let esm = write(&root, "esm.mts", "export {};\n");
    write(&root, "esm.mjs", "export {};\n");
    let common = write(&root, "common.cts", "export {};\n");
    write(&root, "common.cjs", "export {};\n");
    let index = write(&root, "folder/index.ts", "export {};\n");
    let vue = write(&root, "Widget.vue", "<template />\n");
    let absolute = model.to_str().unwrap();
    let cases = [
        ("./model.js", Some(model.as_path())),
        ("./esm.mjs", Some(esm.as_path())),
        ("./common.cjs", Some(common.as_path())),
        ("./folder", Some(index.as_path())),
        ("./Widget.vue", Some(vue.as_path())),
        ("./missing", None),
        (absolute, Some(model.as_path())),
    ];
    for (specifier, expected) in cases {
        let legacy = resolve_local::<true>(
            &importer,
            specifier,
            &mut CanonicalPathCache::default(),
            options(),
            None,
        );
        let untracked = resolve_local::<false>(
            &importer,
            specifier,
            &mut CanonicalPathCache::default(),
            options(),
            None,
        );
        assert_eq!(legacy.0.as_deref(), expected, "{specifier}");
        assert_eq!(untracked.0, legacy.0, "{specifier}");
        assert!(
            !legacy.1.is_empty(),
            "the old path actually constructs history: {specifier}"
        );
        assert!(
            untracked.1.is_empty(),
            "discarded paths must not be constructed: {specifier}"
        );
    }
}

#[test]
fn untracked_path_mode_retains_complete_alias_probe_histories() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let importer = write(&root, "entry.ts", "export {};\n");
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["missing/*","real/*"],"@/special":["specific.ts"]}}}"#,
    );
    let widget = write(&root, "real/component.vue", "<template />\n");
    let special = write(&root, "specific.ts", "export {};\n");
    let base = write(&root, "base.ts", "export {};\n");
    let aliases = PathAliasResolver::from_tsconfig(Some(&config));
    for (specifier, expected) in [
        ("@/component", Some(widget.as_path())),
        ("@/special", Some(special.as_path())),
        ("base", Some(base.as_path())),
        ("@/absent", None),
    ] {
        let legacy = resolve_local::<true>(
            &importer,
            specifier,
            &mut CanonicalPathCache::default(),
            options(),
            Some(&aliases),
        );
        let untracked = resolve_local::<false>(
            &importer,
            specifier,
            &mut CanonicalPathCache::default(),
            options(),
            Some(&aliases),
        );
        assert_eq!(legacy.0.as_deref(), expected, "{specifier}");
        assert_eq!(untracked.0, legacy.0, "{specifier}");
        assert!(!legacy.1.is_empty(), "{specifier}");
        assert_eq!(
            untracked.1, legacy.1,
            "alias proof paths must remain exact: {specifier}"
        );
        if specifier == "@/component" {
            assert_eq!(
                legacy.1.first(),
                Some(&root.join("missing/component")),
                "the first fallback target is the exact missing candidate"
            );
        }
    }
}

#[test]
fn untracked_graphs_keep_exact_work_cycles_and_fresh_missing_to_created_inputs() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    write(
        &root,
        "package.json",
        r#"{"name":"cold-registration-control"}"#,
    );
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["missing/*","real/*"]}}}"#,
    );
    let entry = write(
        &root,
        "entry.ts",
        "import './a';\nimport './model.js';\nimport '@/component';\n",
    );
    write(&root, "a.ts", "export * from './b';\n");
    write(&root, "b.ts", "export * from './a';\n");
    write(&root, "model.ts", "export const model = 1;\n");
    let aliases = PathAliasResolver::from_tsconfig(Some(&config));
    let package = route(&root, entry);
    for created in [false, true] {
        if created {
            write(&root, "real/component.vue", "<template />\n");
        }
        let legacy = scan_route_reachability::<true>(
            &package,
            &mut ReachabilitySources::untracked(),
            &mut CanonicalPathCache::default(),
            options(),
            Some(&aliases),
            &mut vize_canon::PackageRouteResolver::default(),
        );
        let untracked = scan_route_reachability::<false>(
            &package,
            &mut ReachabilitySources::untracked(),
            &mut CanonicalPathCache::default(),
            options(),
            Some(&aliases),
            &mut vize_canon::PackageRouteResolver::default(),
        );
        assert_eq!(untracked.outcome, legacy.outcome);
        assert_eq!(untracked.work, legacy.work);
        assert!(legacy.inputs.is_empty());
        assert!(untracked.inputs.is_empty());
        assert_eq!(
            untracked.outcome,
            if created {
                ReachabilityOutcome::ReachesVue
            } else {
                ReachabilityOutcome::DoesNotReachVue
            }
        );
    }
}

#[test]
fn tracked_graph_control_still_retains_missing_candidates_and_real_sources() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    write(
        &root,
        "package.json",
        r#"{"name":"cold-registration-control"}"#,
    );
    let entry = write(
        &root,
        "entry.ts",
        "import './model.js';\nimport './missing';\n",
    );
    let model = write(&root, "model.ts", "export const model = 1;\n");
    let tracked = scan_route_reachability::<true>(
        &route(&root, entry),
        &mut ReachabilitySources::default(),
        &mut CanonicalPathCache::default(),
        options(),
        None,
        &mut vize_canon::PackageRouteResolver::default(),
    );
    assert_eq!(tracked.outcome, ReachabilityOutcome::DoesNotReachVue);
    assert!(tracked.inputs.iter().any(|path| path == &model));
    assert!(
        tracked
            .inputs
            .iter()
            .any(|path| path == &root.join("missing.ts"))
    );
}

#[test]
fn cross_program_alias_agreement_keeps_equal_winners_with_distinct_proof_paths_apart() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    write(
        &root,
        "package.json",
        r#"{"name":"cold-registration-control"}"#,
    );
    let entry = write(&root, "entry.ts", "import '@/component';\n");
    write(&root, "real/component.vue", "<template />\n");
    let a = write(
        &root,
        "a.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["missing-a/*","real/*"]}}}"#,
    );
    let b = write(
        &root,
        "b.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["missing-b/*","real/*"]}}}"#,
    );
    let aliases = [
        PathAliasResolver::from_tsconfig(Some(&a)),
        PathAliasResolver::from_tsconfig(Some(&b)),
    ];
    let package = route(&root, entry);
    fn history<const RECORD_INPUTS: bool>(
        package: &vize_canon::PackageRoute,
        aliases: &[PathAliasResolver; 2],
    ) -> Vec<usize> {
        let mut sources = ReachabilitySources::untracked();
        let mut counts = Vec::new();
        for (scope, index) in [(1, 0), (2, 1), (3, 0)] {
            sources.set_alias_scope(scope);
            let result = scan_route_reachability::<RECORD_INPUTS>(
                package,
                &mut sources,
                &mut CanonicalPathCache::default(),
                options(),
                Some(&aliases[index]),
                &mut vize_canon::PackageRouteResolver::default(),
            );
            assert_eq!(result.outcome, ReachabilityOutcome::ReachesVue);
            assert!(result.inputs.is_empty());
            counts.push(sources.finished_scans());
        }
        counts
    }
    let legacy = history::<true>(&package, &aliases);
    let untracked = history::<false>(&package, &aliases);
    assert_eq!(
        legacy,
        [1, 2, 2],
        "equal winners do not erase distinct alias probe proofs"
    );
    assert_eq!(
        untracked, legacy,
        "cross-program scan agreement must be unchanged"
    );
}

mod options;
