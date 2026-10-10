use super::ServerState;
use vize_l0::config::TypeCheckerConfig;

#[test]
fn native_checker_snapshot_retains_raw_values_and_separate_origin_on_reload() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("vize.config.json");
    let config = TypeCheckerConfig {
        strict: true,
        tsconfig: Some("app/tsconfig.json".into()),
        tsgo_path: Some("tools/corsa".into()),
        globals_file: Some("globals.d.ts".into()),
        servers: Some(2),
        ..TypeCheckerConfig::default()
    };
    for load in [
        ServerState::load_workspace_config,
        ServerState::load_lsp_config,
    ] {
        let state = ServerState::new();
        assert_eq!(
            state.native_checker_settings(),
            (TypeCheckerConfig::default(), 60_000, None)
        );
        std::fs::write(&config_path, r#"{"typeChecker":{"strict":true,"tsconfig":"app/tsconfig.json","corsaPath":"tools/corsa","globalsFile":"globals.d.ts","servers":2,"lspRequestTimeoutMs":91000}}"#).unwrap();
        load(&state, directory.path());
        assert_eq!(
            state.native_checker_settings(),
            (config.clone(), 91_000, Some(config_path.clone()))
        );
        std::fs::write(&config_path, "{}").unwrap();
        load(&state, directory.path());
        assert_eq!(
            state.native_checker_settings(),
            (
                TypeCheckerConfig::default(),
                60_000,
                Some(config_path.clone())
            )
        );
    }
}

#[test]
fn concurrent_native_checker_reads_keep_values_timeout_and_origin_together() {
    let state = ServerState::new();
    let first = (
        TypeCheckerConfig {
            strict: true,
            tsconfig: Some("first.json".into()),
            ..TypeCheckerConfig::default()
        },
        91_000,
        std::path::PathBuf::from("first/vize.config.json"),
    );
    let second = (
        TypeCheckerConfig {
            strict: false,
            tsconfig: Some("second.json".into()),
            ..TypeCheckerConfig::default()
        },
        92_000,
        std::path::PathBuf::from("second/vize.config.json"),
    );
    state.install_type_checker_snapshot(first.0.clone(), first.1, &first.2);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for index in 0..2_000 {
                let packet = if index % 2 == 0 { &second } else { &first };
                state.install_type_checker_snapshot(packet.0.clone(), packet.1, &packet.2);
            }
        });
        for _ in 0..2_000 {
            let observed = state.native_checker_settings();
            let expected = if observed.0.strict { &first } else { &second };
            assert_eq!(
                observed,
                (expected.0.clone(), expected.1, Some(expected.2.clone()))
            );
        }
    });
}
