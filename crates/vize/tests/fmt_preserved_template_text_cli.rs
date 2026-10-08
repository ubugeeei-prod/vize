#![cfg(feature = "glyph")]
use std::{fs, process::Command};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-preserved-template-text-7871/App.vue.txt"
);
const JSON: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-preserved-template-text-7871/vize.config.json"
);
const TS: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-preserved-template-text-7871/vize.config.ts.txt"
);

#[test]
fn discovered_and_explicit_compiler_whitespace_survives_three_cli_writes() {
    for (config_name, config, explicit) in [
        ("vize.config.json", JSON, false),
        ("vize.config.ts", TS, false),
        ("explicit.json", JSON, true),
        ("explicit.ts", TS, true),
    ] {
        let project = tempfile::tempdir().unwrap();
        fs::write(project.path().join(config_name), config).unwrap();
        fs::write(project.path().join("App.vue"), SOURCE).unwrap();
        for mode in ["--check", "--write", "--write", "--write", "--check"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command.current_dir(project.path()).arg("fmt");
            if explicit {
                command.args(["--config", config_name]);
            }
            let output = command.args([mode, "App.vue"]).output().unwrap();
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stdout, b"");
            assert_eq!(
                fs::read_to_string(project.path().join("App.vue")).unwrap(),
                SOURCE
            );
            assert_eq!(
                fs::read_to_string(project.path().join(config_name)).unwrap(),
                config
            );
        }
    }
}

#[test]
fn no_config_and_explicit_condense_keep_the_existing_formatter() {
    for no_config in [true, false] {
        let project = tempfile::tempdir().unwrap();
        let config = if no_config {
            JSON
        } else {
            "{\"compiler\":{\"whitespace\":\"condense\"}}\n"
        };
        fs::write(project.path().join("vize.config.json"), config).unwrap();
        fs::write(project.path().join("App.vue"), SOURCE).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command.current_dir(project.path()).arg("fmt");
        if no_config {
            command.arg("--no-config");
        }
        let output = command.args(["--write", "App.vue"]).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"");
        assert_eq!(
            fs::read_to_string(project.path().join("App.vue")).unwrap(),
            "<template>\n  <button type=\"button\"> two  spaces </button>\n</template>\n"
        );
    }
}
