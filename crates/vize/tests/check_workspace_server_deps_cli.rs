//! A workspace package's private dependencies resolve identically in every
//! Corsa server mode, including dependencies reached through a package link (#9201).
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{path::Path, process::Command};

use crate::support::corsa_path;
use crate::support::corsa_requirement;

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, content).unwrap();
}

fn run_check(app: &Path, corsa: &str, servers: Option<&str>) -> serde_json::Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(app).env("CORSA_PATH", corsa).args([
        "check",
        "src",
        "--no-config",
        "--format",
        "json",
    ]);
    if let Some(servers) = servers {
        command.args(["--servers", servers]);
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if let Some(directory) = std::env::var_os("VIZE_NATIVE_STRICT_CAPTURE_DIR") {
        static SEQUENCE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let directory = Path::new(&directory).join("workspace-cli");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join(vize_l0::cstr!("{sequence:02}.json").as_str()),
            serde_json::to_vec_pretty(&serde_json::json!({
                "cwd": app, "servers": servers, "exitCode": output.status.code(),
                "wholeStdout": stdout, "wholeStderr": stderr,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("invalid check output: {error}\n{stdout}\n{stderr}"));
    assert_eq!(
        output.status.success(),
        result["errorCount"] == 0,
        "{stdout}\n{stderr}"
    );
    result
}

#[test]
fn workspace_private_dependency_and_its_errors_survive_single_multi_and_auto_servers() {
    check_workspace_server_deps(3);
}

#[test]
fn workspace_private_dependency_and_its_errors_survive_automatic_sharding_threshold() {
    // Automatic sharding starts at 128 Vue files on a multi-core runner. The
    // minimal three-SFC case only exercises the small-project auto fallback.
    check_workspace_server_deps(128);
}

fn check_workspace_server_deps(vue_files: usize) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root))
    else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let root = root.as_path();
    write_file(root, "package.json", r#"{"private":true}"#);
    write_file(
        root,
        "app/package.json",
        r#"{"name":"app","type":"module","dependencies":{"lib":"workspace:*"}}"#,
    );
    write_file(
        root,
        "lib/package.json",
        r#"{"name":"lib","type":"module","exports":{".":"./src/index.ts"},"dependencies":{"private-dep":"1.0.0"}}"#,
    );
    write_file(
        root,
        "app/tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","target":"ES2022","skipLibCheck":true,"noEmit":true},"include":["src/**/*.ts","src/**/*.vue"]}"#,
    );
    write_file(
        root,
        "lib/node_modules/private-dep/package.json",
        r#"{"name":"private-dep","version":"1.0.0","types":"index.d.ts"}"#,
    );
    write_file(
        root,
        "lib/node_modules/private-dep/index.d.ts",
        "export declare function createThing(): string;\n",
    );
    write_file(
        root,
        "lib/src/index.ts",
        "import { createThing } from 'private-dep'\nexport const thing = createThing()\n",
    );
    write_file(
        root,
        "app/src/main.ts",
        "import { thing } from 'lib'\nexport default thing\n",
    );
    for name in 0..vue_files {
        write_file(
            root,
            vize_l0::cstr!("app/src/{name}.vue").as_str(),
            "<template><div /></template>",
        );
    }
    std::fs::create_dir_all(root.join("app/node_modules")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("lib"), root.join("app/node_modules/lib")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(root.join("lib"), root.join("app/node_modules/lib")).unwrap();

    let app = root.join("app");
    for broken in [false, true] {
        if broken {
            // Imported dependencies must retain their real semantic errors.
            write_file(
                root,
                "lib/src/index.ts",
                "import { createThing } from 'private-dep'\nexport const thing: number = createThing()\n",
            );
        }
        let expected = expected_report(root, vue_files, broken);
        for servers in [Some("1"), Some("2"), None] {
            assert_eq!(
                run_check(&app, &corsa, servers),
                expected,
                "complete report: broken={broken}, servers={servers:?}"
            );
        }
    }
}

// The fixture authors every input and owns the entire report, including empty
// diagnostic receivers. Server agreement alone could agree on the same omission.
fn expected_report(root: &Path, vue_files: usize, broken: bool) -> serde_json::Value {
    let mut source_files: Vec<_> = (0..vue_files)
        .map(|index| vize_l0::cstr!("src/{index}.vue"))
        .collect();
    source_files.push("src/main.ts".into());
    source_files.sort();
    let mut files = vec![serde_json::json!({
        "file": root.join("lib/src/index.ts"),
        "diagnostics": if broken {
            vec!["error:2:14 [TS2322] Type 'string' is not assignable to type 'number'."]
        } else { vec![] },
    })];
    files.extend(source_files.iter().map(|file| {
        serde_json::json!({
            "file": file.as_str(), "diagnostics": [],
        })
    }));
    // The CLI sorts by displayed path, including the absolute external receiver.
    files.sort_by(|left, right| left["file"].as_str().cmp(&right["file"].as_str()));
    let program_files: Vec<_> = source_files.iter().map(|file| file.as_str()).collect();
    serde_json::json!({
        "files": files,
        "programs": [{
            "root": ".", "tsconfig": "tsconfig.json", "files": program_files,
            "compilerOptions": {
                "strict": true, "module": "ESNext", "moduleResolution": "Bundler",
                "target": "ES2022", "skipLibCheck": true, "noEmit": true,
            },
        }],
        "errorCount": usize::from(broken), "warningCount": 0,
        "fileCount": vue_files + 2,
    })
}
