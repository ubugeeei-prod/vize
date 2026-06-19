use std::path::Path;

use super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use vize_carton::{String, cstr};

#[test]
fn node_module_resolution_uses_package_types_hidden_by_exports() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let Some(snapshot) = package_exports_hidden_types_diagnostics("node") else {
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/main.ts"
                && (*code == Some(7016) || message.contains("package.json \"exports\"")))
        }),
        "legacy Node moduleResolution should use bundled package declarations: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/main.ts"
                && *code == Some(2322)
                && message.contains("Type 'string' is not assignable to type 'number'")
        }),
        "expected package declarations to type imported API as returning string: {snapshot:#?}"
    );
}

#[test]
fn bundler_module_resolution_still_respects_package_exports() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let Some(snapshot) = package_exports_hidden_types_diagnostics("bundler") else {
        return;
    };

    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/main.ts"
                && *code == Some(7016)
                && message.contains("package.json \"exports\"")
        }),
        "bundler moduleResolution should still respect package exports: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| !(file == "src/main.ts" && *code == Some(2322))),
        "bundler resolution must not load declarations hidden by exports: {snapshot:#?}"
    );
}

fn package_exports_hidden_types_diagnostics(
    module_resolution: &str,
) -> Option<Vec<(String, Option<u32>, String)>> {
    let project_root = create_project_case(
        cstr!("package-exports-hidden-types-{module_resolution}").as_str(),
        &[(
            "src/main.ts",
            r#"import hiddenTypes from "exports-hidden-types";

const ok: string = hiddenTypes.parse("ok");
const wrong: number = hiddenTypes.parse("typed");

void ok;
void wrong;
"#,
        )],
    );
    write_tsconfig(&project_root, module_resolution);
    write_exports_hidden_types_package(&project_root);

    let result = (|| {
        let mut checker = BatchTypeChecker::new(&project_root).ok()?;
        checker.scan_project().ok()?;
        let result = checker.check_project().ok()?;
        let mut snapshot: Vec<_> = result
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                (
                    relative_path(&project_root, &diagnostic.file),
                    diagnostic.code,
                    cstr!(
                        "{}:{}:{} {}",
                        diagnostic.line + 1,
                        diagnostic.column + 1,
                        match diagnostic.severity {
                            1 => "error",
                            2 => "warning",
                            3 => "info",
                            _ => "hint",
                        },
                        diagnostic.message
                    ),
                )
            })
            .collect();
        snapshot.sort();
        Some(snapshot)
    })();

    let _ = std::fs::remove_dir_all(&project_root);
    result
}

fn write_tsconfig(project_root: &Path, module_resolution: &str) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        cstr!(
            r#"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "{module_resolution}",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
        ),
    )
    .unwrap();
}

fn write_exports_hidden_types_package(project_root: &Path) {
    let package_dir = project_root.join("node_modules/exports-hidden-types");
    std::fs::create_dir_all(package_dir.join("lib")).unwrap();
    std::fs::create_dir_all(package_dir.join("types")).unwrap();
    std::fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "exports-hidden-types",
  "version": "1.0.0",
  "main": "./lib/main.js",
  "types": "./types/index.d.ts",
  "exports": {
    ".": "./lib/main.js",
    "./package.json": "./package.json"
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        package_dir.join("lib/main.js"),
        "module.exports = { parse(value) { return String(value); } };\n",
    )
    .unwrap();
    std::fs::write(
        package_dir.join("types/index.d.ts"),
        r#"declare const api: {
  parse(value: string): string;
};
export default api;
"#,
    )
    .unwrap();
}
