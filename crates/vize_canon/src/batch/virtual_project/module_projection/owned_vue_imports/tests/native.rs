//! Mandatory native discriminator; observations are saved before assertions.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::{CHILD, packet, project, register};
use crate::batch::virtual_project::VirtualProject;
use crate::batch::{BatchTypeChecker, DeclarationEmitOptions};

fn save(directory: &Path, name: &str, value: serde_json::Value) {
    std::fs::create_dir_all(directory).unwrap();
    std::fs::write(
        directory.join(name),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    eprintln!("owned_vue_native {name}: {value}");
}

fn native_check(native: &Path, config: &Path, directory: &Path, phase: &str) -> Output {
    let result = Command::new(native)
        .args(["--project"])
        .arg(config)
        .args(["--pretty", "false"])
        .output();
    let value = match &result {
        Ok(output) => serde_json::json!({"native": native, "configPath": config,
            "config": std::fs::read_to_string(config).unwrap(),
            "status": output.status.code(), "stdout": output.stdout, "stderr": output.stderr}),
        Err(error) => {
            serde_json::json!({"native": native, "configPath": config, "error": error.to_string()})
        }
    };
    save(directory, &format!("{phase}.json"), value);
    result.unwrap()
}

fn snapshot(project: &VirtualProject, directory: &Path, phase: &str) {
    save(
        directory,
        &format!("{phase}-inputs.json"),
        serde_json::json!({
            "config": std::fs::read_to_string(project.generated_tsconfig_path()).unwrap(),
            "files": project.virtual_files_sorted().into_iter().map(|file| {
                serde_json::json!({"original": file.original_path, "virtual": file.virtual_path,
                    "content": file.content, "map": format!("{:?}", file.source_map),
                    "authored": project.original_contents.get(&file.virtual_path),
                    "pristine": project.pre_rewrite_code.get(&file.virtual_path)})
            }).collect::<Vec<_>>()
        }),
    );
}

fn link_node_modules(root: &Path) {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(workspace.join("node_modules"), root.join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(workspace.join("node_modules"), root.join("node_modules"))
        .unwrap();
}

#[test]
fn owned_relative_imports_preserve_native_diagnostics_and_declarations() {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").is_ok_and(|value| value == "1");
    let disabled = std::env::var("VIZE_TEST_DISABLE_TSGO").is_ok_and(|value| value == "1");
    let native = std::env::var_os("CORSA_PATH")
        .map(PathBuf::from)
        .filter(|path| path.is_file());
    if disabled || native.is_none() {
        assert!(
            !required,
            "owned Vue native test requires the existing native provider"
        );
        return;
    }
    let native = native.unwrap();
    let capture = std::env::var_os("VIZE_OWNED_VUE_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("owned-vue-native-{}", std::process::id()))
        });
    for (mode, module) in [("bundler", "ESNext"), ("Node16", "Node16")] {
        let root = tempfile::tempdir().unwrap();
        link_node_modules(root.path());
        std::fs::write(root.path().join("package.json"), "{\"type\":\"module\"}").unwrap();
        let options = serde_json::json!({"compilerOptions":{"strict":true,"target":"ES2022",
            "module":module,"moduleResolution":mode,"types":[],"noEmit":true},
            "include":["Child.vue","Parent.vue","authored.ts","plain.ts"]});
        let mut project = project(root.path(), &options.to_string());
        register(&mut project, "Child.vue", CHILD);
        let good = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child :count=\"1\" /></template>";
        let bad = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child count=\"wrong\" /></template>";
        for (phase, source, expected) in [("clean", good, 0), ("prop-error", bad, 1)] {
            register(&mut project, "Parent.vue", source);
            project.finalize_module_imports();
            project.materialize().unwrap();
            let phase = format!("{mode}-{phase}");
            snapshot(&project, &capture, &phase);
            let output = native_check(
                &native,
                &project.generated_tsconfig_path(),
                &capture,
                &phase,
            );
            let text = String::from_utf8(output.stdout).unwrap();
            assert_eq!(
                output.status.code(),
                Some(if expected == 0 { 0 } else { 2 }),
                "{text}"
            );
            assert!(output.stderr.is_empty());
            assert!(
                !text.contains("TS5097"),
                "generated import must not require allowImportingTsExtensions: {text}"
            );
            assert_eq!(text.matches("error TS2322:").count(), expected, "{text}");
            assert_eq!(text.matches("error TS").count(), expected, "{text}");
            assert!(
                packet(&project, "Parent.vue")["code"]
                    .as_str()
                    .unwrap()
                    .contains("'./Child.vue.js'")
            );
        }
        register(&mut project, "Parent.vue", good);
        register(&mut project, "plain.ts", "export const value = 1;\n");
        register(
            &mut project,
            "authored.ts",
            "import { value } from './plain.ts'; export const wrong: string = value;\n",
        );
        project.finalize_module_imports();
        project.materialize().unwrap();
        let phase = format!("{mode}-authored-control");
        snapshot(&project, &capture, &phase);
        let output = native_check(
            &native,
            &project.generated_tsconfig_path(),
            &capture,
            &phase,
        );
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(2), "{text}");
        assert!(output.stderr.is_empty());
        assert_eq!(text.matches("error TS5097:").count(), 1, "{text}");
        assert_eq!(text.matches("error TS2322:").count(), 1, "{text}");
        assert_eq!(text.matches("error TS").count(), 2, "{text}");
        assert!(
            text.lines()
                .filter(|line| line.contains("error TS"))
                .all(|line| line.contains("authored.ts")),
            "{text}"
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("authored.ts")).unwrap(),
            "import { value } from './plain.ts'; export const wrong: string = value;\n"
        );
    }

    // Exercise the existing public declaration emitter and its real consumer,
    // including its declaration postprocessing, instead of a synthetic emitter.
    let root = tempfile::tempdir().unwrap();
    link_node_modules(root.path());
    let options = r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","types":[],"noEmit":true},"include":["Child.vue","Parent.vue"]}"#;
    let mut project = project(root.path(), options);
    register(
        &mut project,
        "Child.vue",
        "<script lang=\"ts\">export interface Props { count: number }; export default {};</script><template />",
    );
    register(
        &mut project,
        "Parent.vue",
        "<script lang=\"ts\">export { default as Child } from './Child.vue'; export type { Props } from './Child.vue';</script><template />",
    );
    let mut checker = BatchTypeChecker::new(root.path()).unwrap();
    checker.scan_project().unwrap();
    let out = root.path().join("types");
    let emitted = checker.emit_declarations(&DeclarationEmitOptions::new(out.clone()));
    save(
        &capture,
        "declaration-result.json",
        serde_json::json!({"result":format!("{emitted:?}")}),
    );
    let emitted = emitted.unwrap();
    let parent = emitted
        .files
        .iter()
        .find(|file| {
            file.path
                .file_name()
                .is_some_and(|name| name == "Parent.vue.d.ts")
        })
        .unwrap();
    let directory = parent.path.parent().unwrap();
    std::fs::write(directory.join("consumer.ts"), "import type { Props } from './Parent.vue.js'; export const wrong: Props = { count: 'wrong' };\n").unwrap();
    let config = directory.join("tsconfig.consumer.json");
    std::fs::write(&config, r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","types":[],"noEmit":true},"files":["consumer.ts"]}"#).unwrap();
    save(
        &capture,
        "declaration-inputs-and-outputs.json",
        serde_json::json!({"files":checker.virtual_files().into_iter().map(|file|serde_json::json!({"original":file.original_path,"virtual":file.virtual_path,"code":file.content,"map":format!("{:?}",file.source_map)})).collect::<Vec<_>>(),"outputs":emitted.files.iter().map(|file|serde_json::json!({"path":file.path,"content":file.content})).collect::<Vec<_>>()}),
    );
    let output = native_check(&native, &config, &capture, "declaration-consumer");
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(2), "{text}");
    assert!(output.stderr.is_empty());
    assert_eq!(text.matches("error TS2322:").count(), 1, "{text}");
    assert_eq!(text.matches("error TS").count(), 1, "{text}");
    assert!(
        text.lines()
            .filter(|line| line.contains("error TS"))
            .all(|line| line.contains("consumer.ts")),
        "{text}"
    );
}
