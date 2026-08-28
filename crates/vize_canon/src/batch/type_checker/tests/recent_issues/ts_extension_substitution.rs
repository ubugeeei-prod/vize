use super::super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use vize_s0::{String, cstr};

const SFC: &str = "<script setup lang=\"ts\">\nconst local = 1\n</script>\n<template><div>{{ local }}</div></template>\n";

#[test]
fn explicit_vue_ts_imports_follow_typescript_extension_substitution() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    for sfc_first in [false, true] {
        let case_name = if sfc_first {
            "vue-ts-extension-substitution-sfc-first"
        } else {
            "vue-ts-extension-substitution-authored-first"
        };
        let project_root = create_project_case(
            case_name,
            &[
                (
                    "src/App.vue",
                    r#"<script setup lang="ts">
import { typed } from "./Typed.vue.ts";
import { runtime } from "./Runtime.vue.ts";
import { full } from "./Full.vue.ts";
import { directory } from "./Directory.vue.ts";
import { direct } from "./Direct.vue.ts";
import { packaged } from "./Packaged.vue.ts";
import { main } from "./Main.vue.ts";

const typedNumber: number = typed;
const runtimeNumber: number = runtime;
const fullNumber: number = full;
const directoryNumber: number = directory;
const directNumber: number = direct;
const packagedNumber: number = packaged;
const mainNumber: number = main;
</script>
"#,
                ),
                (
                    "src/Typed.vue.d.ts",
                    "export declare const typed: string;\n",
                ),
                ("src/Runtime.vue.js", "export const runtime = 'runtime';\n"),
                (
                    "src/Full.vue.ts.ts",
                    "export const full: string = 'full';\n",
                ),
                (
                    "src/Directory.vue.ts/index.ts",
                    "export const directory: string = 'directory';\n",
                ),
                (
                    "src/Direct.vue.ts",
                    "export const direct: string = 'direct';\n",
                ),
                (
                    "src/Packaged.vue.ts/package.json",
                    r#"{"types":"./types.d.ts"}"#,
                ),
                (
                    "src/Packaged.vue.ts/types.d.ts",
                    "export declare const packaged: string;\n",
                ),
                ("src/Main.vue.ts/package.json", r#"{"main":"./entry.js"}"#),
                (
                    "src/Main.vue.ts/entry.d.ts",
                    "export declare const main: string;\n",
                ),
                ("src/Main.vue.ts/entry.js", "export const main = 'main';\n"),
                ("src/Typed.vue", SFC),
                ("src/Runtime.vue", SFC),
                ("src/Full.vue", SFC),
                ("src/Directory.vue", SFC),
                ("src/Direct.vue", SFC),
                ("src/Packaged.vue", SFC),
                ("src/Main.vue", SFC),
            ],
        );

        let app = project_root.join("src/App.vue");
        let mut scan_paths = [
            "Typed.vue",
            "Runtime.vue",
            "Full.vue",
            "Directory.vue",
            "Direct.vue",
            "Packaged.vue",
            "Main.vue",
        ]
        .map(|name| project_root.join("src").join(name))
        .to_vec();
        let authored_direct = project_root.join("src/Direct.vue.ts");
        if sfc_first {
            scan_paths.push(authored_direct);
            scan_paths.push(app);
        } else {
            scan_paths.insert(0, authored_direct);
            scan_paths.insert(0, app);
        }

        let mut checker = BatchTypeChecker::new(&project_root).unwrap();
        checker.scan_paths(&scan_paths).unwrap();
        let result = checker.check_project().unwrap();
        let virtual_root = crate::batch::project_virtual_root(&project_root);
        let virtual_root = virtual_root.to_string_lossy();
        let project_prefix = project_root.to_string_lossy();
        let mut snapshot: Vec<_> = result
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                // `<virtual>` must never appear: since #3227 a diagnostic body
                // names the authored root, not the materialized mirror. Both
                // substitutions stay so the assertion below distinguishes the
                // two rather than accepting whichever one is produced.
                let message = diagnostic
                    .message
                    .replace(virtual_root.as_ref(), "<virtual>")
                    .replace(project_prefix.as_ref(), "<project>");
                (
                    relative_path(&project_root, &diagnostic.file),
                    diagnostic.code,
                    cstr!(
                        "{}:{} {}",
                        diagnostic.line + 1,
                        diagnostic.column + 1,
                        message
                    ),
                )
            })
            .collect();
        snapshot.sort();

        assert_eq!(
            snapshot,
            vec![
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("10:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("12:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("13:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("14:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("15:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(2322),
                    String::from("16:7 Type 'string' is not assignable to type 'number'."),
                ),
                (
                    String::from("src/App.vue"),
                    Some(7016),
                    String::from(
                        "3:25 Could not find a declaration file for module './Runtime.vue.ts'. '<project>/src/Runtime.vue.ts.__vize_authored_vue_ts_alias__.js' implicitly has an 'any' type."
                    ),
                ),
            ]
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }
}
