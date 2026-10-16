//! A workspace package's private dependencies resolve identically in every
//! Corsa server mode, including dependencies reached through a package link (#9201).
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{path::Path, process::Command};

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

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
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root))
    else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
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
    for name in ["A", "B", "C"] {
        write_file(
            root,
            &format!("app/src/{name}.vue"),
            "<template><div /></template>",
        );
    }
    std::fs::create_dir_all(root.join("app/node_modules")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("lib"), root.join("app/node_modules/lib")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(root.join("lib"), root.join("app/node_modules/lib")).unwrap();

    let app = root.join("app");
    for servers in [Some("1"), Some("2"), None] {
        let result = run_check(&app, &corsa, servers);
        assert_eq!(result["errorCount"], 0, "servers={servers:?}: {result}");
        assert_eq!(result["fileCount"], 5, "servers={servers:?}: {result}");
    }

    // Preserve real semantic errors in the imported dependency; fixing the
    // false positive must not silently drop the external file's diagnostics.
    write_file(
        root,
        "lib/src/index.ts",
        "import { createThing } from 'private-dep'\nexport const thing: number = createThing()\n",
    );
    let expected = run_check(&app, &corsa, Some("1"));
    assert_eq!(expected["errorCount"], 1, "{expected}");
    assert!(expected.to_string().contains("TS2322"), "{expected}");
    for servers in [Some("2"), None] {
        assert_eq!(
            run_check(&app, &corsa, servers),
            expected,
            "servers={servers:?}"
        );
    }
}
