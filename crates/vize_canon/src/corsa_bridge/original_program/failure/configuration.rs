//! Inherited options can change while the authored source/root config stay exact.

use super::*;

#[test]
fn inherited_option_change_at_real_lsp_startup_refuses_and_reaps_original_checks() {
    for lang in [Lang::Js, Lang::Ts] {
        let root = tempfile::TempDir::new().unwrap();
        let base = root.path().join("base.json");
        let initial = r#"{"compilerOptions":{"strict":false,"allowJs":true,"checkJs":true,"moduleDetection":"force","types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"}}"#;
        let changed = initial.replace("\"strict\":false", "\"strict\":true");
        std::fs::write(&base, initial).unwrap();
        let config = root.path().join("tsconfig.json");
        let configuration = r#"{"extends":"./base.json","include":["source.mjs","source.ts"]}"#;
        std::fs::write(&config, configuration).unwrap();
        let path = root.path().join(if lang == Lang::Js {
            "source.mjs"
        } else {
            "source.ts"
        });
        // Under false this is clean; the changed diagnosing strict option
        // produces actual TS18047. Returning old options with that report lies.
        let source = "const value=null; value.toFixed();";
        std::fs::write(&path, source).unwrap();
        let changed_file = root.path().join("changed.json");
        std::fs::write(&changed_file, &changed).unwrap();
        let pid_path = root.path().join("overlay.pid");
        let wrapper = root.path().join("tsc");
        write_executable(
            &wrapper,
            &cstr!(
                "#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then\n echo \"$$\" > {}\n cat {} > {}\nfi\nexec {} \"$@\"\n",
                shell_quote(&pid_path),
                shell_quote(&changed_file),
                shell_quote(&base),
                shell_quote(&backend())
            ),
        );
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(wrapper),
            working_dir: Some(root.path().to_path_buf()),
            timeout_ms: 30000,
            ..Default::default()
        });
        block_on(bridge.spawn()).unwrap();
        let arena = Allocator::default();
        let original = file(&arena, source, lang);
        let result = block_on(bridge.check_original_program(&original, &path));
        assert!(matches!(
            result,
            Err(OriginalProgramError::ConfigurationChanged)
        ));
        assert!(original.is_complete(), "{:?}", original.issues());
        assert_eq!(original.artifact().source(), source);
        assert_eq!(std::fs::read(&config).unwrap(), configuration.as_bytes());
        assert_eq!(std::fs::read(&base).unwrap(), changed.as_bytes());
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        let pid: i32 = std::fs::read_to_string(&pid_path)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // The check has returned only after its real diagnosing owner was reaped.
        // SAFETY: signal zero only observes this owned exec child's PID.
        assert_ne!(
            unsafe { libc::kill(pid, 0) },
            0,
            "diagnosing process survived refusal"
        );
        block_on(bridge.shutdown()).unwrap();
    }
}
