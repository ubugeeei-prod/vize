//! Mandatory native discriminator; observations are saved before assertions.

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sha2::{Digest, Sha256};
use vize_carton::{String, ToCompactString, cstr, line_index::LineBreaks};

use super::{CHILD, packet, project, register};
use crate::batch::virtual_project::VirtualProject;
use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, DeclarationEmitOptions};

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
    let cwd = config.parent().unwrap();
    let arguments = ["--project", config.to_str().unwrap(), "--pretty", "false"];
    let authority_environment = serde_json::json!({
        "CORSA_PATH": std::env::var("CORSA_PATH").ok(),
        "VIZE_TEST_REQUIRE_TSGO": std::env::var("VIZE_TEST_REQUIRE_TSGO").ok(),
        "VIZE_TEST_DISABLE_TSGO": std::env::var("VIZE_TEST_DISABLE_TSGO").ok(),
        "remainingEnvironment": "inherited and not captured; no environment-independence claim"
    });
    let result = Command::new(native)
        .current_dir(cwd)
        .args(["--project"])
        .arg(config)
        .args(["--pretty", "false"])
        .output();
    let value = match &result {
        Ok(output) => {
            #[cfg(unix)]
            let signal = output.status.signal();
            #[cfg(not(unix))]
            let signal: Option<i32> = None;
            serde_json::json!({"native": native, "argv":arguments, "cwd":cwd,
                "authorityEnvironment":authority_environment, "configPath": config,
                "config": std::fs::read_to_string(config).unwrap(),
                "status": output.status.code(), "signal":signal,
                "stdout": output.stdout, "stderr": output.stderr})
        }
        Err(error) => {
            serde_json::json!({"native": native, "argv":arguments, "cwd":cwd,
                "authorityEnvironment":authority_environment, "configPath": config,
                "error": error.to_compact_string()})
        }
    };
    save(directory, &cstr!("{phase}.json"), value);
    result.unwrap()
}

fn snapshot(project: &VirtualProject, directory: &Path, phase: &str) {
    save(
        directory,
        &cstr!("{phase}-inputs.json"),
        serde_json::json!({
            "config": std::fs::read_to_string(project.generated_tsconfig_path()).unwrap(),
            "files": project.virtual_files_sorted().into_iter().map(|file| {
                serde_json::json!({"original": file.original_path, "virtual": file.virtual_path,
                    "content": file.content, "map": cstr!("{:?}", file.source_map),
                    "authored": project.original_contents.get(&file.virtual_path),
                    "pristine": project.pre_rewrite_code.get(&file.virtual_path)})
            }).collect::<Vec<_>>()
        }),
    );
}

fn native_image(native: &Path) -> serde_json::Value {
    let bytes = std::fs::read(native).unwrap();
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    serde_json::json!({"binary":native,"canonicalBinary":native.canonicalize().unwrap(),
        "bytes":bytes.len(),"sha256":digest})
}

fn expected_diagnostic(
    config: &Path,
    file: &Path,
    source: &str,
    needle: &str,
    code: u32,
    message: &str,
) -> String {
    assert_eq!(source.matches(needle).count(), 1);
    let offset = source.find(needle).unwrap();
    let (line, column) = LineBreaks::Lsp.offset_to_position(source, offset);
    let relative = file.strip_prefix(config.parent().unwrap()).unwrap();
    cstr!(
        "{}({},{}): error TS{code}: {message}\n",
        relative.display(),
        line + 1,
        column + 1
    )
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
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let native = match std::env::var_os("CORSA_PATH") {
        Some(path) => Some(PathBuf::from(path)).filter(|path| path.is_file()),
        None => vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace),
    };
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
            std::env::temp_dir().join(cstr!("owned-vue-native-{}", std::process::id()).as_str())
        });
    let before = native_image(&native);
    save(&capture, "provider-before.json", before.clone());
    let version = Command::new(&native).arg("--version").output();
    let version_packet = match &version {
        Ok(output) => serde_json::json!({"provider":before,"argv":["--version"],
            "status":output.status.code(),"stdout":output.stdout,"stderr":output.stderr}),
        Err(error) => {
            serde_json::json!({"provider":before,"argv":["--version"],"error":error.to_compact_string()})
        }
    };
    save(&capture, "provider-version.json", version_packet);
    assert!(version.unwrap().status.success());
    for (mode, module) in [("bundler", "ESNext"), ("Node16", "Node16")] {
        let root = tempfile::tempdir().unwrap();
        link_node_modules(root.path());
        std::fs::write(root.path().join("package.json"), "{\"type\":\"module\"}").unwrap();
        let options = serde_json::json!({"compilerOptions":{"strict":true,"target":"ES2022",
            "module":module,"moduleResolution":mode,"types":[],"noEmit":true},
            "include":["Child.vue","Parent.vue","authored.ts","plain.ts"]});
        let mut project = project(root.path(), &options.to_compact_string());
        register(&mut project, "Child.vue", CHILD);
        let good = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child :count=\"1\" /></template>";
        let bad = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child count=\"wrong\" /></template>";
        for (phase, source, wrong_prop) in [("clean", good, false), ("prop-error", bad, true)] {
            register(&mut project, "Parent.vue", source);
            let before = packet(&project, "Parent.vue");
            project.finalize_module_imports();
            let after = packet(&project, "Parent.vue");
            let mut expected_projection = before.clone();
            expected_projection["code"] = before["code"]
                .as_str()
                .unwrap()
                .replace("'./Child.vue.ts'", "'./Child.vue.js'")
                .into();
            assert_eq!(after, expected_projection);
            assert_ne!(after["code"], before["code"]);
            project.materialize().unwrap();
            let phase = cstr!("{mode}-{phase}");
            snapshot(&project, &capture, &phase);
            let output = native_check(
                &native,
                &project.generated_tsconfig_path(),
                &capture,
                &phase,
            );
            let text = std::str::from_utf8(&output.stdout).unwrap();
            assert_eq!(
                output.status.code(),
                Some(if wrong_prop { 1 } else { 0 }),
                "{text}"
            );
            assert!(output.stderr.is_empty());
            let expected_stdout = if wrong_prop {
                let parent = project
                    .find_by_original(&root.path().join("Parent.vue"))
                    .unwrap();
                let config = project.generated_tsconfig_path();
                let message = "Type 'string' is not assignable to type 'number'.";
                // These are distinct producer-owned checks: the value annotation
                // and the whole-props checker. Keep both complete diagnostics.
                let mut expected = expected_diagnostic(
                    &config,
                    &parent.virtual_path,
                    &parent.content,
                    "__vize_prop_check_0_count:",
                    2322,
                    message,
                );
                expected.push_str(
                    expected_diagnostic(
                        &config,
                        &parent.virtual_path,
                        &parent.content,
                        r#""count": "wrong""#,
                        2322,
                        message,
                    )
                    .as_str(),
                );
                expected
            } else {
                String::new()
            };
            assert_eq!(text, expected_stdout.as_str());
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
        let phase = cstr!("{mode}-authored-control");
        snapshot(&project, &capture, &phase);
        let output = native_check(
            &native,
            &project.generated_tsconfig_path(),
            &capture,
            &phase,
        );
        let text = std::str::from_utf8(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{text}");
        assert!(output.stderr.is_empty());
        let authored = project
            .find_by_original(&root.path().join("authored.ts"))
            .unwrap();
        let config = project.generated_tsconfig_path();
        let mut expected = expected_diagnostic(
            &config,
            &authored.virtual_path,
            &authored.content,
            "'./plain.ts'",
            5097,
            "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.",
        );
        expected.push_str(
            expected_diagnostic(
                &config,
                &authored.virtual_path,
                &authored.content,
                "wrong: string",
                2322,
                "Type 'number' is not assignable to type 'string'.",
            )
            .as_str(),
        );
        assert_eq!(text, expected.as_str());
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
    project.finalize_module_imports();
    project.materialize().unwrap();
    let direct_out = root.path().join("direct-native-types");
    let direct_config = project
        .write_declaration_tsconfig(&direct_out, false)
        .unwrap();
    snapshot(&project, &capture, "direct-declaration");
    let output = native_check(&native, &direct_config, &capture, "direct-declaration");
    let direct_outputs = ["Child.vue.d.ts", "Parent.vue.d.ts"].map(|name| {
        let path = direct_out.join(name);
        let content = std::fs::read_to_string(&path);
        let observed = match &content {
            Ok(code) => serde_json::json!({"path":path,"content":code}),
            Err(error) => serde_json::json!({"path":path,"error":error.to_compact_string()}),
        };
        (content.is_ok(), observed)
    });
    save(
        &capture,
        "direct-declaration-outputs.json",
        serde_json::json!(
            direct_outputs
                .iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>()
        ),
    );
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(output.stderr.is_empty());
    assert_eq!(text, "");
    assert!(direct_outputs.iter().all(|(present, _)| *present));

    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        root.path(),
        BatchTypeCheckerOptions::default(),
        Some(&native),
    )
    .unwrap();
    checker.scan_project().unwrap();
    let out = root.path().join("types");
    save(
        &capture,
        "public-emitter-inputs.json",
        serde_json::json!({
            "scope":"Actual public API inputs; delegated internal transport is not captured",
            "root":root.path(), "outDir":out, "native":native,
            "files":checker.virtual_files().into_iter().map(|file|serde_json::json!({
                "original":file.original_path,"virtual":file.virtual_path,
                "code":file.content,"map":cstr!("{:?}",file.source_map)
            })).collect::<Vec<_>>()
        }),
    );
    let emitted = checker.emit_declarations(&DeclarationEmitOptions::new(out.clone()));
    save(
        &capture,
        "declaration-result.json",
        serde_json::json!({"result":cstr!("{emitted:?}"),
            "scope":"Actual public API result; existing executor may lossily decode/trim failure streams; not a raw transport packet"}),
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
    let consumer_source = "import type { Props } from './Parent.vue.js'; export const wrong: Props = { count: 'wrong' };\n";
    std::fs::write(directory.join("consumer.ts"), consumer_source).unwrap();
    let config = directory.join("tsconfig.consumer.json");
    std::fs::write(&config, r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","types":[],"noEmit":true},"files":["consumer.ts"]}"#).unwrap();
    save(
        &capture,
        "declaration-inputs-and-outputs.json",
        serde_json::json!({"consumer":{"path":directory.join("consumer.ts"),"source":consumer_source},"files":checker.virtual_files().into_iter().map(|file|serde_json::json!({"original":file.original_path,"virtual":file.virtual_path,"code":file.content,"map":cstr!("{:?}",file.source_map)})).collect::<Vec<_>>(),"outputs":emitted.files.iter().map(|file|serde_json::json!({"path":file.path,"content":file.content})).collect::<Vec<_>>()}),
    );
    let output = native_check(&native, &config, &capture, "declaration-consumer");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(output.stderr.is_empty());
    let expected = expected_diagnostic(
        &config,
        &directory.join("consumer.ts"),
        consumer_source,
        "count: 'wrong'",
        2322,
        "Type 'string' is not assignable to type 'number'.",
    );
    assert_eq!(text, expected.as_str());
    let after = native_image(&native);
    save(&capture, "provider-after.json", after.clone());
    assert_eq!(after, before);
}
