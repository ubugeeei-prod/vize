use super::{CompilerInputs, ConfigurationObservations, read_jsonc};
use crate::package_route::stamp::InputStamp;
use crate::{CompilerConfigurationSnapshot, CorsaError, snapshot_tsconfig_compiler_options};

#[test]
fn owned_observations_preserve_original_read_and_absent_candidate_identity() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConfigurationObservations>();
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let base = root.join("base.json");
    let absent = root.join("candidate.json");
    std::fs::write(&base, "{}").unwrap();
    let inputs = CompilerInputs::default();
    inputs.read_jsonc(&base).unwrap();
    inputs.observe(&absent);
    let observed = inputs.observations();
    drop(inputs);
    assert!(observed.changed_paths().is_empty());
    let mut expected = vec![base.clone(), absent.clone()];
    expected.sort();
    assert_eq!(observed.paths(), expected);
    std::fs::write(&base, r#"{"strict":true}"#).unwrap();
    std::fs::write(&absent, "{}").unwrap();
    assert_eq!(observed.changed_paths(), expected);
    let repair_only = ConfigurationObservations::capture(&observed.paths());
    assert!(repair_only.changed_paths().is_empty());
    assert_eq!(observed.changed_paths(), expected);
    let inputs = CompilerInputs::default();
    inputs.observe(&root.join("base.json"));
    inputs.mark_changed();
    assert!(inputs.observations().changed_paths().is_empty());
    assert!(inputs.observations().needs_confirmation());
}

#[test]
fn validation_aggregates_keep_duplicate_roles_conflicts_and_fresh_reads() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("base.json");
    let absent = directory.path().join("absent.json");
    let content = b"alpha";
    std::fs::write(&file, content).unwrap();
    let physical = InputStamp::capture(&file);
    let source = InputStamp::capture_source(&file, content);
    let mut observed = ConfigurationObservations {
        stamps: vec![
            physical.clone(),
            physical,
            source.clone(),
            source,
            InputStamp::capture(&absent),
        ],
        changed_during_capture: false,
    };
    let (paths, stats, unique) = observed.observed_changed_paths();
    assert!(paths.is_empty());
    assert_eq!(observed.stamps.len(), 5);
    assert_eq!(unique, 3);
    assert_eq!(stats.reads, 2);
    assert_eq!(stats.bytes, 10);
    assert_eq!(
        paths,
        observed.collect_changed_paths(InputStamp::is_current)
    );

    // A later validation must read disk again, even for the same paths/roles.
    std::fs::write(&file, b"omega").unwrap();
    assert_eq!(observed.changed_paths(), vec![file.clone()]);
    let conflicting = ConfigurationObservations::capture(std::slice::from_ref(&file));
    observed.extend(&conflicting);
    let (paths, stats, unique) = observed.observed_changed_paths();
    assert_eq!(paths, vec![file.clone()]);
    assert_eq!(observed.stamps.len(), 6);
    assert_eq!(unique, 3);
    assert_eq!(stats.reads, 2);
    assert_eq!(stats.bytes, 10);
    assert_eq!(
        paths,
        observed.collect_changed_paths(InputStamp::is_current)
    );
    observed.stamps.reverse();
    assert_eq!(observed.changed_paths(), paths);
    observed.stamps.reverse();
    assert!(conflicting.observed_changed_paths().0.is_empty());
    std::fs::write(&absent, content).unwrap();
    let (paths, stats, unique) = observed.observed_changed_paths();
    let mut expected = vec![file, absent];
    expected.sort();
    assert_eq!(paths, expected);
    assert_eq!(unique, 3);
    assert_eq!(stats.reads, 3);
    assert_eq!(stats.bytes, 15);
}

#[cfg(unix)]
#[test]
fn validation_aggregates_preserve_symlink_source_and_filesystem_roles() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target.json");
    let link = directory.path().join("base.json");
    let content = b"alpha";
    std::fs::write(&target, content).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let observed = ConfigurationObservations {
        stamps: vec![
            InputStamp::capture(&link),
            InputStamp::capture_source(&link, content),
        ],
        changed_during_capture: false,
    };
    let (paths, stats, unique) = observed.observed_changed_paths();
    assert!(paths.is_empty());
    assert_eq!(unique, 2);
    assert_eq!(stats.reads, 1);
    assert_eq!(stats.bytes, 5);
    std::fs::write(&target, b"omega").unwrap();
    assert!(observed.stamps[0].is_current());
    assert!(!observed.stamps[1].is_current());
    let (paths, stats, _) = observed.observed_changed_paths();
    assert_eq!(paths, vec![link]);
    assert_eq!(
        paths,
        observed.collect_changed_paths(InputStamp::is_current)
    );
    assert_eq!(stats.reads, 1);
    assert_eq!(stats.bytes, 5);
}

#[test]
fn cached_and_live_parse_errors_keep_the_consumed_file_and_complete_parser_reason() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let path = root.join("tsconfig.base.json");
    std::fs::write(&path, "{\n").unwrap();
    let expected = vize_carton::cstr!(
        "Configuration error: cannot parse tsconfig {}: JSON parse error: EOF while parsing an object at line 2 column 0",
        path.display()
    );
    let inputs = CompilerInputs::default();
    for error in [
        read_jsonc(&path, Some(&inputs)).unwrap_err(),
        read_jsonc(&path, None).unwrap_err(),
    ] {
        assert!(matches!(error, CorsaError::Configuration(_)));
        assert_eq!(vize_carton::cstr!("{error}"), expected);
    }
    std::fs::write(
        &path,
        "{/* repaired */ \"compilerOptions\": {\"strict\": true,},}\n",
    )
    .unwrap();
    // A capture keeps its original consumed revision even after physical repair.
    let error = read_jsonc(&path, Some(&inputs)).unwrap_err();
    assert_eq!(vize_carton::cstr!("{error}"), expected);
    assert!(inputs.observations().needs_confirmation());
    let repaired = serde_json::json!({"compilerOptions": {"strict": true}});
    assert_eq!(read_jsonc(&path, None).unwrap(), repaired);
    assert_eq!(
        CompilerInputs::default().read_jsonc(&path).unwrap(),
        repaired
    );
}

#[test]
fn malformed_inherited_file_is_named_by_both_admitted_and_live_compiler_consumers() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let app = root.join("app");
    std::fs::create_dir(&app).unwrap();
    let config = app.join("tsconfig.json");
    let base = root.join("tsconfig.base.json");
    let source =
        r#"{"extends":"../tsconfig.base.json","compilerOptions":{"strictNullChecks":false}}"#;
    std::fs::write(&config, source).unwrap();
    std::fs::write(&base, "{invalid").unwrap();
    let expected = vize_carton::cstr!(
        "Configuration error: cannot parse tsconfig {}: JSON parse error: key must be a string at line 1 column 2",
        base.display()
    );
    let attempt = CompilerConfigurationSnapshot::capture_attempt(&app, Some(&config));
    let expected_dependencies = vec![config.clone(), base.clone()];
    assert_eq!(attempt.dependencies, expected_dependencies);
    for error in [
        attempt.outcome.err().unwrap(),
        snapshot_tsconfig_compiler_options(&app, &config).unwrap_err(),
    ] {
        assert!(matches!(error, CorsaError::Configuration(_)));
        assert_eq!(vize_carton::cstr!("{error}"), expected);
    }
    std::fs::write(
        &base,
        "{/* inherited */ \"compilerOptions\": {\"strict\": true,},}\n",
    )
    .unwrap();
    let admitted = CompilerConfigurationSnapshot::capture(&app, Some(&config)).unwrap();
    assert_eq!(admitted.dependencies(), expected_dependencies);
    assert_eq!(
        snapshot_tsconfig_compiler_options(&app, &config).unwrap(),
        serde_json::json!({"strict":true,"strictNullChecks":false})
            .as_object()
            .unwrap()
            .clone()
    );
    assert_eq!(std::fs::read_to_string(&config).unwrap(), source);
}
