use super::ServerState;
use vize_l0::config::TypeCheckerConfig;

#[test]
fn native_checker_snapshot_retains_one_project_model_on_reload() {
    let directory = crate::test_support::tempdir().unwrap();
    let config_path = directory.path().join("vite.config.mjs");
    std::fs::create_dir(directory.path().join("app")).unwrap();
    std::fs::write(directory.path().join("app/tsconfig.json"), "{}").unwrap();
    let config = TypeCheckerConfig {
        strict: true,
        tsconfig: Some(
            directory
                .path()
                .join("app/tsconfig.json")
                .to_str()
                .unwrap()
                .into(),
        ),
        tsgo_path: Some(
            directory
                .path()
                .join("tools/corsa")
                .to_str()
                .unwrap()
                .into(),
        ),
        globals_file: Some(
            directory
                .path()
                .join("globals.d.ts")
                .to_str()
                .unwrap()
                .into(),
        ),
        servers: Some(2),
        ..TypeCheckerConfig::default()
    };
    for load in [
        ServerState::load_workspace_config,
        ServerState::load_lsp_config,
    ] {
        let state = ServerState::new();
        let (initial_config, initial_timeout, initial_project) = state.native_checker_settings();
        assert_eq!(initial_config, TypeCheckerConfig::default());
        assert_eq!(initial_timeout, 60_000);
        assert_eq!(initial_project.root(), None);
        assert_eq!(initial_project.config_source(), None);
        assert_eq!(initial_project.tsconfig(), None);
        std::fs::write(&config_path, r#"export default { vue: {"typeChecker":{"strict":true,"tsconfig":"app/tsconfig.json","corsaPath":"tools/corsa","globalsFile":"globals.d.ts","servers":2,"lspRequestTimeoutMs":91000}} };"#).unwrap();
        load(&state, directory.path());
        let (loaded_config, timeout, project) = state.native_checker_settings();
        assert_eq!(loaded_config, config);
        assert_eq!(timeout, 91_000);
        assert_eq!(project.root(), Some(directory.path()));
        assert_eq!(project.config_source(), Some(config_path.as_path()));
        assert_eq!(
            project.tsconfig(),
            config.tsconfig.as_deref().map(std::path::Path::new)
        );
        std::fs::write(&config_path, "export default { vue: {} };").unwrap();
        load(&state, directory.path());
        let (default_config, timeout, project) = state.native_checker_settings();
        assert_eq!(default_config, TypeCheckerConfig::default());
        assert_eq!(timeout, 60_000);
        assert_eq!(project.root(), Some(directory.path()));
        assert_eq!(project.config_source(), Some(config_path.as_path()));
        assert_eq!(project.tsconfig(), None);
    }
}

#[test]
fn concurrent_native_checker_reads_keep_values_timeout_and_project_together() {
    let state = ServerState::new();
    let first = (
        TypeCheckerConfig {
            strict: true,
            tsconfig: Some("first.json".into()),
            ..TypeCheckerConfig::default()
        },
        91_000,
        std::path::PathBuf::from("first"),
        std::path::PathBuf::from("first/vite.config.mjs"),
    );
    let second = (
        TypeCheckerConfig {
            strict: false,
            tsconfig: Some("second.json".into()),
            ..TypeCheckerConfig::default()
        },
        92_000,
        std::path::PathBuf::from("second"),
        std::path::PathBuf::from("second/vite.config.mjs"),
    );
    state.install_type_checker_snapshot(&first.2, first.0.clone(), first.1, Some(&first.3));
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for index in 0..2_000 {
                let packet = if index % 2 == 0 { &second } else { &first };
                state.install_type_checker_snapshot(
                    &packet.2,
                    packet.0.clone(),
                    packet.1,
                    Some(&packet.3),
                );
            }
        });
        for _ in 0..2_000 {
            let observed = state.native_checker_settings();
            let expected = if observed.0.strict { &first } else { &second };
            assert_eq!(observed.0, expected.0);
            assert_eq!(observed.1, expected.1);
            assert_eq!(observed.2.root(), Some(expected.2.as_path()));
            assert_eq!(observed.2.config_source(), Some(expected.3.as_path()));
        }
    });
}
