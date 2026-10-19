//! Shared repair baselines retain every owner's independent Canon authority.
use super::config_reload::ConfigurationPostRead;
use super::freshness::ConfigurationObservation;
use super::{LoadedLspConfig, ProjectContextKey, ServerState};
use vize_canon::ConfigurationObservations;

fn fixture() -> std::io::Result<(tempfile::TempDir, LoadedLspConfig)> {
    let fixture = tempfile::tempdir()?;
    let raw = super::super::config::host::load_lsp_config_snapshot(Some(fixture.path()));
    Ok((fixture, raw))
}

#[test]
fn shared_post_read_keeps_earliest_common_revision_and_each_owners_specific_inputs() {
    let (fixture, mut raw) = fixture().unwrap();
    let common = fixture.path().join("policy.mjs");
    let absent = fixture.path().join("optional.mjs");
    let a = fixture.path().join("a.json");
    let b = fixture.path().join("b.json");
    for path in [&common, &a, &b] {
        std::fs::write(path, "{}").unwrap();
    }
    raw.watch_dependencies = vec![common.clone(), absent.clone(), common.clone()];
    let mut shared = ConfigurationPostRead::new(&raw.watch_dependencies);
    // Construction performs no read; the first successful owner's boundary
    // captures the current repair baseline, never evaluator-consumed bytes.
    std::fs::write(&common, "[]").unwrap();
    let before = ConfigurationObservations::default();
    let mut applied = raw.clone();
    applied.watch_dependencies.push(a.clone());
    let first = ConfigurationObservation::checked(
        &raw,
        &applied,
        &before,
        &ConfigurationObservations::capture(std::slice::from_ref(&a)),
        Some(&mut shared),
    );
    assert!(first.changes().is_empty());
    assert!(first.needs_confirmation());
    std::fs::write(&common, "{}").unwrap();
    std::fs::write(&absent, "{}").unwrap();
    std::fs::write(&a, "[]").unwrap();
    applied.watch_dependencies = raw.watch_dependencies.clone();
    applied.watch_dependencies.push(b.clone());
    let second = ConfigurationObservation::checked(
        &raw,
        &applied,
        &before,
        &ConfigurationObservations::capture(std::slice::from_ref(&b)),
        Some(&mut shared),
    );
    let mut changed = vec![common.clone(), absent.clone()];
    changed.sort();
    assert_eq!(second.changes(), changed);
    assert!(!second.paths().contains(&a));
    assert!(second.paths().contains(&b));
    changed.push(a);
    changed.sort();
    assert_eq!(first.changes(), changed);
    // A new attempt must read again, with no shared sample across attempts.
    let mut next = ConfigurationPostRead::new(&raw.watch_dependencies);
    let current = ConfigurationObservation::checked(
        &raw,
        &applied,
        &before,
        &ConfigurationObservations::capture(&[b]),
        Some(&mut next),
    );
    assert!(current.changes().is_empty());
    assert!(current.needs_confirmation());
}

#[test]
fn shared_vite_baseline_preserves_full_key_compiler_graphs_and_failed_repair() {
    let (fixture, mut raw) = fixture().unwrap();
    let root = super::paths::physical_path(fixture.path());
    let common = root.join("policy.mjs");
    let base = root.join("base.json");
    let leaf_base = root.join("leaf-base.json");
    let leaf = root.join("leaf.json");
    std::fs::write(&common, "export default {};").unwrap();
    std::fs::write(&base, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
    std::fs::write(&leaf_base, r#"{"vueCompilerOptions":{"target":3}}"#).unwrap();
    std::fs::write(root.join("tsconfig.json"), r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&leaf, r#"{"extends":"./leaf-base.json"}"#).unwrap();
    raw.watch_dependencies = vec![
        common.clone(),
        root.join("optional-helper.mjs"),
        common.clone(),
    ];
    let before = ConfigurationObservations::capture(&raw.watch_dependencies);
    let mut shared = ConfigurationPostRead::new(&raw.watch_dependencies);
    let base_key = ProjectContextKey::from(root.clone());
    let leaf_key = ProjectContextKey {
        root,
        tsconfig: Some(leaf.clone()),
    };
    let (base_snapshot, _) = ServerState::prepare_configuration_attempt(
        &base_key,
        &raw,
        &before,
        &[],
        Some(&mut shared),
    );
    let (leaf_snapshot, _) = ServerState::prepare_configuration_attempt(
        &leaf_key,
        &raw,
        &before,
        &[],
        Some(&mut shared),
    );
    let base_snapshot = base_snapshot.unwrap();
    let leaf_snapshot = leaf_snapshot.unwrap();
    for snapshot in [&base_snapshot, &leaf_snapshot] {
        // Real apply begins with raw.clone and only appends Canon paths. Keep
        // the complete raw scope, including its order, duplicates and absence.
        assert!(
            snapshot
                .applied
                .watch_dependencies
                .starts_with(&raw.watch_dependencies)
        );
        assert_eq!(snapshot.authored.watch_dependencies, raw.watch_dependencies);
    }
    assert!(base_snapshot.applied.watch_dependencies.contains(&base));
    assert!(
        leaf_snapshot
            .applied
            .watch_dependencies
            .contains(&leaf_base)
    );
    assert_eq!(
        base_snapshot.compiler.vue_target().unwrap(),
        Some(vize_l0::config::VueVersion::V2_7)
    );
    assert_eq!(
        leaf_snapshot.compiler.vue_target().unwrap(),
        Some(vize_l0::config::VueVersion::V3)
    );
    assert!(base_snapshot.compiler.dependencies().contains(&base));
    assert!(!base_snapshot.compiler.dependencies().contains(&leaf_base));
    assert!(leaf_snapshot.compiler.dependencies().contains(&leaf_base));
    assert!(!leaf_snapshot.compiler.dependencies().contains(&base));
    std::fs::write(&base, "{}").unwrap();
    assert_eq!(base_snapshot.observation.changes(), vec![base]);
    assert!(leaf_snapshot.observation.changes().is_empty());
    std::fs::remove_file(&leaf_base).unwrap();
    let (rejected, failed) = ServerState::prepare_configuration_attempt(
        &leaf_key,
        &raw,
        &before,
        &[],
        Some(&mut shared),
    );
    assert!(rejected.is_err());
    assert!(failed.rejected);
    assert!(failed.paths().contains(&leaf_base));
    std::fs::write(&leaf_base, r#"{"vueCompilerOptions":{"target":3}}"#).unwrap();
    assert_eq!(failed.changes(), vec![leaf_base]);
    let mut next = ConfigurationPostRead::new(&raw.watch_dependencies);
    let (repaired, _) =
        ServerState::prepare_configuration_attempt(&leaf_key, &raw, &before, &[], Some(&mut next));
    assert!(repaired.is_ok());
    std::fs::write(&common, "export default []; ").unwrap();
    let (rejected, failed) =
        ServerState::prepare_configuration_attempt(&leaf_key, &raw, &before, &[], Some(&mut next));
    assert_eq!(
        rejected.err().unwrap().as_str(),
        "project configuration changed during evaluation; keeping the last good configuration"
    );
    assert!(failed.rejected);
    assert!(failed.needs_confirmation());
}

#[test]
fn common_repair_baseline_never_overwrites_a_later_canon_consumed_revision() {
    let (fixture, mut raw) = fixture().unwrap();
    let root = super::paths::physical_path(fixture.path());
    let base = root.join("base.json");
    let leaf = root.join("leaf.json");
    std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(&base, r#"{"vueCompilerOptions":{"target":3}}"#).unwrap();
    std::fs::write(&leaf, r#"{"extends":"./base.json"}"#).unwrap();
    raw.watch_dependencies = vec![base.clone()];
    let before = ConfigurationObservations::default();
    let mut shared = ConfigurationPostRead::new(&raw.watch_dependencies);
    let key = ProjectContextKey::from(root.clone());
    let (first, _) =
        ServerState::prepare_configuration_attempt(&key, &raw, &before, &[], Some(&mut shared));
    assert!(first.is_ok());
    std::fs::write(&base, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
    let key = ProjectContextKey {
        root,
        tsconfig: Some(leaf),
    };
    let (next, _) =
        ServerState::prepare_configuration_attempt(&key, &raw, &before, &[], Some(&mut shared));
    let next = next.unwrap();
    assert_eq!(
        next.compiler.vue_target().unwrap(),
        Some(vize_l0::config::VueVersion::V2_7)
    );
    assert_eq!(next.observation.changes(), vec![base]);
    assert!(next.observation.needs_confirmation());
}

#[cfg(unix)]
#[test]
fn shared_post_read_retains_symlink_route_changes_between_owners() {
    let (fixture, mut raw) = fixture().unwrap();
    let first = fixture.path().join("one");
    let second = fixture.path().join("two");
    let alias = fixture.path().join("alias");
    for directory in [&first, &second] {
        std::fs::create_dir(directory).unwrap();
        std::fs::write(directory.join("policy.mjs"), "export default {};").unwrap();
    }
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    raw.watch_dependencies = vec![alias.join("policy.mjs")];
    let mut shared = ConfigurationPostRead::new(&raw.watch_dependencies);
    let empty = ConfigurationObservations::default();
    let checked = |shared: &mut ConfigurationPostRead| {
        ConfigurationObservation::checked(&raw, &raw, &empty, &empty, Some(shared))
    };
    assert!(checked(&mut shared).changes().is_empty());
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(second, &alias).unwrap();
    assert!(checked(&mut shared).changes().contains(&alias));
    assert!(
        checked(&mut ConfigurationPostRead::new(&raw.watch_dependencies))
            .changes()
            .is_empty()
    );
}
