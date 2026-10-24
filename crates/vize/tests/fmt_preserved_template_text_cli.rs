#![cfg(feature = "glyph")]
#![expect(
    clippy::disallowed_macros,
    reason = "historical JavaScript config carrier"
)]
use crate::support::vite_config;

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
        ("vite.config.mjs", JSON, false),
        ("vite.config.mjs", TS, false),
        ("vite.config.mts", JSON, true),
        ("vite.config.ts", TS, true),
    ] {
        let project = tempfile::tempdir().unwrap();
        let config = if let Some(native) = config.strip_prefix("export default ") {
            let native = native.trim().trim_end_matches(';');
            format!("export default {{ vue: {native} }};\n")
        } else {
            vite_config::source(config)
        };
        fs::write(project.path().join(config_name), &config).unwrap();
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
        fs::write(
            project.path().join("vite.config.mjs"),
            vite_config::source(config),
        )
        .unwrap();
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

#[test]
fn vue2_and_entry_whitespace_keep_preserved_compile_output_across_formatting() {
    let project = tempfile::tempdir().unwrap();
    let config = vite_config::source(
        r#"{
  "compiler": { "whitespace": "vue2-preserve" },
  "entries": [
    { "files": ["src/**"], "compiler": { "whitespace": "condense" } },
    { "files": ["src/legacy/**"], "compiler": { "whitespace": "preserve" } }
  ]
}"#,
    );
    fs::create_dir_all(project.path().join("src/legacy")).unwrap();
    fs::write(project.path().join("vite.config.mjs"), config).unwrap();
    for file in ["Root.vue", "src/Modern.vue", "src/legacy/Legacy.vue"] {
        fs::write(project.path().join(file), SOURCE).unwrap();
    }

    let build = |output: &str| {
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("RAYON_NUM_THREADS", "1")
            .args([
                "build",
                "--slow-threshold",
                "600000",
                "-o",
                output,
                "-f",
                "js",
                "Root.vue",
                "src/Modern.vue",
                "src/legacy/Legacy.vue",
            ])
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
    };
    build("before");

    let formatted = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--write",
            "Root.vue",
            "src/Modern.vue",
            "src/legacy/Legacy.vue",
        ])
        .output()
        .unwrap();
    assert!(formatted.status.success(), "{formatted:?}");
    assert_eq!(
        fs::read_to_string(project.path().join("Root.vue")).unwrap(),
        SOURCE
    );
    assert_eq!(
        fs::read_to_string(project.path().join("src/legacy/Legacy.vue")).unwrap(),
        SOURCE
    );
    assert_ne!(
        fs::read_to_string(project.path().join("src/Modern.vue")).unwrap(),
        SOURCE
    );

    build("after");
    for file in ["Root.js", "src/legacy/Legacy.js"] {
        assert_eq!(
            fs::read(project.path().join("before").join(file)).unwrap(),
            fs::read(project.path().join("after").join(file)).unwrap(),
            "formatting changed the compiled output of {file}"
        );
    }
    assert_ne!(
        fs::read(project.path().join("before/src/Modern.js")).unwrap(),
        fs::read(project.path().join("after/src/Modern.js")).unwrap(),
        "the condense entry must retain the existing formatter path"
    );
}
