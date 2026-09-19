use super::*;

#[test]
fn inline_exports_emit_usable_declarations_for_a_separate_consumer() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    for newline in ["\n", "\r\n"] {
        let source = "<script lang=\"ts\">\nconst emoji = '😀'; export /* kept */ const café = 1; export class Counter { value = café; }\nconst counter: Counter = new Counter();\n</script>\n<template><div /></template>".replace('\n', newline);
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
        let source = "import { café, Counter } from './lib/App.vue';\nconst value: number = café;\nconst counter: Counter = new Counter();\nconst precise: number = counter.value;\n";
        let consumer = create_cli_project("inline-export-consumer", &[("src/index.ts", source)]);
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
            let target = consumer.join("src/lib").join(relative);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(library.join(file), target).unwrap();
        }
        assert!(check_consumer(&consumer, &corsa).status.success());
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
        std::fs::write(consumer.join("src/index.ts"), source).unwrap();
        assert!(check_consumer(&consumer, &corsa).status.success());
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
