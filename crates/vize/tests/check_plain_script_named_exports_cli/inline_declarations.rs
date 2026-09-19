use super::*;

#[test]
fn inline_exports_emit_usable_declarations_for_a_separate_consumer() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    for newline in ["\n", "\r\n"] {
        let source = "<script lang=\"ts\">\nconst emoji = '😀'; export /* kept */ const café = 1; export class Counter { value = café; }\nexport enum Mode { One, Two }\nexport enum Color { Red = 'red', Blue = 'blue' }\nexport const enum Flag { Enabled = 1 }\nexport enum Merged { First = 1 }\nexport enum Merged { Second = 2 }\nexport namespace Merged { export const label = 'merged'; }\nexport class Box<T> { private brand = 0; constructor(public value: T) {} }\nconst counter: Counter = new Counter();\n</script>\n<template><div /></template>".replace('\n', newline);
        let library = create_cli_project("inline-export-library", &[("src/App.vue", &source)]);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&library)
            .env("CORSA_PATH", &corsa)
            .args([
                "check",
                ".",
                "--format",
                "json",
                "--declaration",
                "--declaration-dir",
                "types",
            ])
            .output()
            .unwrap();
        let emitted = successful_json(&output);
        let source = "import { café, Counter, Mode, Color, Flag, Merged, Box } from './lib/App.vue';\nconst mode: Mode = Mode.One;\nconst color: Color = Color.Red;\nconst flag: Flag = Flag.Enabled;\nconst merged: Merged = Merged.Second;\nconst label: string = Merged.label;\nconst box: Box<number> = new Box(1);\nconst boxed: number = box.value;\nimport { café as packed } from 'alpha-lib/App.vue';\nconst packedValue: number = packed;\nconst value: number = café;\nconst counter: Counter = new Counter();\nconst precise: number = counter.value;\n";
        let consumer = create_cli_project(
            "inline-export-consumer",
            &[
                ("src/index.ts", source),
                (
                    "src/Consumer.vue",
                    "<script setup lang=\"ts\">const value: number = 1;</script><template>{{ value }}</template>",
                ),
                (
                    "node_modules/alpha-lib/package.json",
                    r#"{"name":"alpha-lib","exports":{"./App.vue":{"types":"./App.vue.d.ts"}}}"#,
                ),
            ],
        );
        let vue = workspace_root()
            .join("playground/node_modules/vue")
            .canonicalize()
            .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, consumer.join("node_modules/vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, consumer.join("node_modules/vue")).unwrap();
        let declarations = emitted["declarations"].as_array().unwrap();
        assert!(!declarations.is_empty(), "{emitted}");
        for file in declarations {
            let file = Path::new(file.as_str().unwrap());
            let relative = file.strip_prefix("types").unwrap();
            for prefix in ["src/lib", "src/another-library", "node_modules/alpha-lib"] {
                let target = consumer.join(prefix).join(relative);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::copy(library.join(file), target).unwrap();
            }
        }
        assert!(check_consumer(&consumer, &corsa).status.success());
        successful_json(&run_check_json(&consumer, &corsa));
        std::fs::write(
            consumer.join("src/index.ts"),
            source.replace("const value: number", "const value: string"),
        )
        .unwrap();
        let broken = check_consumer(&consumer, &corsa);
        assert_eq!(broken.status.code(), Some(1));
        let diagnostic = String::from_utf8_lossy(&broken.stdout);
        assert_eq!(
            diagnostic.matches("error TS2322:").count(),
            1,
            "{diagnostic}"
        );
        let checked = run_check_json(&consumer, &corsa);
        assert_eq!(checked.status.code(), Some(1));
        let json: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
        assert_eq!(json["errorCount"], 1, "{json}");
        assert!(
            String::from_utf8_lossy(&checked.stdout).contains("TS2322"),
            "{json}"
        );
        std::fs::write(consumer.join("src/index.ts"), source).unwrap();
        assert!(check_consumer(&consumer, &corsa).status.success());
        successful_json(&run_check_json(&consumer, &corsa));
        let nominal = format!(
            "{source}\nconst wrong: Color = 'red';\nconst fake: Box<number> = {{ value: 1 }};\n"
        );
        std::fs::write(consumer.join("src/index.ts"), nominal).unwrap();
        let native = check_consumer(&consumer, &corsa);
        assert_eq!(native.status.code(), Some(1));
        let diagnostic = String::from_utf8_lossy(&native.stdout);
        assert_eq!(
            diagnostic.matches("error TS2322:").count(),
            1,
            "{diagnostic}"
        );
        assert_eq!(
            diagnostic.matches("error TS2741:").count(),
            1,
            "{diagnostic}"
        );
        let checked = run_check_json(&consumer, &corsa);
        let json: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
        assert_eq!(json["errorCount"], 2, "{json}");
        std::fs::write(consumer.join("src/index.ts"), source).unwrap();
        let config_path = consumer.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
        config["include"] = serde_json::json!(["src/index.ts", "src/Consumer.vue"]);
        std::fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
        assert!(check_consumer(&consumer, &corsa).status.success());
        successful_json(&run_check_json(&consumer, &corsa));
        std::fs::remove_dir_all(library).unwrap();
        std::fs::remove_dir_all(consumer).unwrap();
    }
}

fn successful_json(output: &std::process::Output) -> serde_json::Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{json}");
    json
}

fn check_consumer(project: &Path, corsa: &str) -> std::process::Output {
    let output = Command::new(corsa)
        .current_dir(project)
        .args(["--project", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}
