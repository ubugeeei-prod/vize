#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-canon-remapped-key-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

#[test]
fn check_remapped_key_indexed_assignment_matches_typescript() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("indexed-assignment");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2023",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/foo.ts"),
        r#"type WithOptionalBooleans<T> = {
  [K in keyof T as [T[K]] extends [boolean] ? K : never]?: T[K];
} & {
  [K in keyof T as [T[K]] extends [boolean] ? never : K]: T[K];
};

export function pickDefinedProps<T extends Record<string, unknown>>(
  source: T,
  key: string
): WithOptionalBooleans<T> {
  const result = {} as WithOptionalBooleans<T>;
  const value = source[
    key
  ] as WithOptionalBooleans<T>[keyof WithOptionalBooleans<T>];

  result[key as keyof WithOptionalBooleans<T>] = value;

  return result;
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8 stdout>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
