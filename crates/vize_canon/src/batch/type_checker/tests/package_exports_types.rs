use std::path::Path;

use super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use vize_carton::{String, cstr};

/// An unresolved SFC import must quote the specifier the author wrote.
///
/// The import rewriter redirects `./Absent.vue` onto the generated mirror
/// module `./Absent.vue.ts`, so before the fix `TS2307` named a path that
/// appears nowhere in the source while `vue-tsc` 3.3.4 reports
/// `Cannot find module './Absent.vue' ...` at the identical position. The
/// second import pins the other direction: a hand-written `./Literal.vue.ts`
/// is not a mirror module and keeps its own spelling.
#[test]
fn unresolved_sfc_import_reports_the_authored_specifier() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "unresolved-sfc-import-specifier",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import Absent from "./Absent.vue";
import Literal from "./Literal.vue.ts";
</script>

<template>
  <Absent />
  <Literal />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/App.vue"),
                Some(2307),
                String::from(
                    "2:20:error Cannot find module './Absent.vue' or its corresponding type declarations."
                ),
            ),
            (
                String::from("src/App.vue"),
                Some(2307),
                String::from(
                    "3:21:error Cannot find module './Literal.vue.ts' or its corresponding type declarations."
                ),
            ),
        ]
    );
}

/// Every message that embeds a module specifier must quote the authored one.
///
/// `TS2614` anchors at the imported member, not at the specifier, so the
/// positional check that fixed `TS2307` in #3397 cannot see the import at all
/// and the generated mirror spelling leaked into both the sentence and the
/// quick-fix suggestion — which named `import Bare from "…/Local.vue.ts"`, a
/// path the user cannot type (#3438). `vue-tsc` 3.3.4 on the same workspace
/// reports, at the identical position:
///
/// ```text
/// src/pages/LocalConsumer.vue(2,10): error TS2614: Module '"../components/Local.vue"' has no exported member 'Bare'. Did you mean to use 'import Bare from "../components/Local.vue"' instead?
/// ```
///
/// The second consumer pins the other direction: a hand-written `.vue.ts`
/// specifier must not resolve through the generated mirror, and its `TS2307`
/// must keep the exact spelling the author wrote (#3482).
#[test]
fn module_member_diagnostics_report_the_authored_specifier() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "module-member-authored-specifier",
        &[
            (
                "src/components/Local.vue",
                r#"<script lang="ts">
namespace Bare {
  export const label = "bare";
}

export default { name: "Local", data: () => ({ label: Bare.label }) };
</script>
"#,
            ),
            (
                "src/pages/LocalConsumer.vue",
                r#"<script setup lang="ts">
import { Bare } from "../components/Local.vue";

const label: string = Bare.label;
</script>
"#,
            ),
            (
                "src/pages/LiteralConsumer.vue",
                r#"<script setup lang="ts">
import { Bare } from "../components/Local.vue.ts";
import { real } from "../components/Authored.vue.ts";
import "../components/Missing.vue.ts/__vize_authored_vue_ts__";
import { directory } from "../components/Directory.vue.ts";

const label: string = Bare.label + real + directory;
</script>
"#,
            ),
            (
                "src/components/Authored.vue.ts",
                r#"export const real: string = "authored";
"#,
            ),
            (
                "src/components/Directory.vue.ts/index.ts",
                "export const directory: string = 'directory';\n",
            ),
            // With `allowArbitraryExtensions`, the old sibling suffix poison
            // resolved this declaration instead of reporting TS2307. The
            // child-path poison cannot traverse the generated mirror file.
            (
                "src/components/Local.vue.ts.d.__vize_authored_vue_ts__.ts",
                "export declare const Bare: { label: number };\n",
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/pages/LiteralConsumer.vue"),
                Some(2307),
                String::from(
                    "2:22:error Cannot find module '../components/Local.vue.ts' or its corresponding type declarations."
                ),
            ),
            // The `Missing.vue.ts/...` side-effect import reports nothing:
            // stable `tsc` does not check side-effect imports unless the
            // project opts into `noUncheckedSideEffectImports`, and the
            // mirror pins that stable default (#4964).
            (
                String::from("src/pages/LocalConsumer.vue"),
                Some(2614),
                String::from(
                    "2:10:error Module '\"../components/Local.vue\"' has no exported member 'Bare'. Did you mean to use 'import Bare from \"../components/Local.vue\"' instead?"
                ),
            ),
        ]
    );
}

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
