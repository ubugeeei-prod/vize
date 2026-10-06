use super::super::{CollectedRoots, split_program_candidates};
use crate::commands::check::{path_cache::CanonicalPathCache, tsconfig_inputs::TsconfigInputCache};
use vize_l0::FxHashSet;

#[test]
fn explicit_workspace_inputs_keep_the_actual_nearest_config_filename() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    let root_config = root.join("tsconfig.json");
    std::fs::write(&root_config, r#"{"files":[]}"#).unwrap();
    let mut files = Vec::new();
    let mut expected = Vec::new();
    for (package, config) in [
        ("javascript", "jsconfig.json"),
        ("typescript", "tsconfig.json"),
    ] {
        let dir = root.join("packages").join(package);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let config = dir.join(config);
        std::fs::write(&config, r#"{"include":["src/**/*"]}"#).unwrap();
        let file = dir.join("src/App.vue");
        std::fs::write(&file, "<template />").unwrap();
        expected.push((config, vec![file.clone()]));
        files.push(file);
    }
    let mut cache = TsconfigInputCache::default();
    let mut paths = CanonicalPathCache::default();
    let candidates = split_program_candidates(
        CollectedRoots {
            reported: FxHashSet::default(),
            package_routes: Vec::new(),
            inputs: files.clone(),
            files,
        },
        Some(&root_config),
        true,
        false,
        &mut cache,
        &mut paths,
    );
    let actual = candidates
        .into_iter()
        .map(|candidate| {
            assert!(candidate.rebuild_supporting_files);
            (candidate.tsconfig_path.unwrap(), candidate.inputs)
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
