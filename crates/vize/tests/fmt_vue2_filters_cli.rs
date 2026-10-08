#![cfg(feature = "glyph")]

use std::{fs, process::Command};

#[test]
fn fmt_uses_explicit_project_vue_version_and_no_config_retains_vue3() {
    let cases = [
        (
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-filter-chain-crlf/App.vue.txt").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-filter-chain-crlf/vize.config.json").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-filter-chain-crlf/sole-child-width.current.expected.txt").as_slice(),
        ),
        (
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-7-filter-chain/App.vue.txt").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-7-filter-chain/vize.config.json").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue2-7-filter-chain/reference.expected.txt").as_slice(),
        ),
        (
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue3-bitwise-or/App.vue.txt").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue3-bitwise-or/vize.config.json").as_slice(),
            include_bytes!("../../../tests/_fixtures/differential/formatter/sfc-vue3-bitwise-or/sole-child-width.current.expected.txt").as_slice(),
        ),
    ];
    for (source, config, expected) in cases {
        let project = tempfile::tempdir().unwrap();
        fs::write(project.path().join("App.vue"), source).unwrap();
        fs::write(project.path().join("vize.config.json"), config).unwrap();
        for _ in 0..3 {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args(["fmt", "--config", "vize.config.json", "--write", "App.vue"])
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
            assert_eq!(fs::read(project.path().join("App.vue")).unwrap(), expected);
        }
    }

    let project = tempfile::tempdir().unwrap();
    let (source, config, _) = cases[1];
    fs::write(project.path().join("App.vue"), source).unwrap();
    fs::write(project.path().join("vize.config.json"), config).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--no-config", "--write", "App.vue"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(
        fs::read(project.path().join("App.vue")).unwrap(),
        cases[2].2
    );
}
