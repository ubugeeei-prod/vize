//! #13001: removing readonly members changes the asserted type.
use super::{corsa_available, failure_context, lint_sfc_with_corsa};
use crate::{LintPreset, Linter, Severity};
use std::path::Path;

const RULE: &str = "type/no-unnecessary-type-assertion";
const MESSAGE: &str =
    "This assertion is unnecessary since it does not change the type of the expression.";

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ES2022", "DOM"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022"
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#;

const IMPORTED: &str = r#"export interface Point {
  x: number;
  y: number;
}
export interface ReadonlyPoint {
  readonly x: number;
  readonly y: number;
}
export const ORIGIN: ReadonlyPoint = { x: 0, y: 0 };
"#;

const SOURCE: &str = r#"<script setup lang="ts">
import { ORIGIN, type Point } from "./types";

interface LocalPoint { x: number }
interface LocalReadonlyPoint { readonly x: number }
type AliasPoint = { x: number };
type AliasReadonlyPoint = { readonly x: number };
class ClassPoint { x = 0 }
class ClassReadonlyPoint { readonly x = 0 }

declare const local: LocalReadonlyPoint;
declare const alias: AliasReadonlyPoint;
declare const instance: ClassReadonlyPoint;
declare const box: DOMRectReadOnly;
declare const stable: number;

const importedDraft = ORIGIN as Point;
const localDraft = local as LocalPoint;
const aliasDraft = alias as AliasPoint;
const classDraft = instance as ClassPoint;
const rect = box as DOMRect;
const identical = stable as number;

importedDraft.x = 1;
localDraft.x = 1;
aliasDraft.x = 1;
classDraft.x = 1;
rect.x = 1;
</script>

<template>
  <p>{{ importedDraft.x }} {{ rect.x }}</p>
</template>
"#;

#[test]
fn readonly_member_removal_is_not_reported_and_identical_packet_is_unchanged() {
    if !corsa_available() {
        return;
    }

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("no-unnecessary-assertion-readonly-13001-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(source_dir.join("types.ts"), IMPORTED).expect("imported types");
    let path = source_dir.join("Draft.vue");
    std::fs::write(&path, SOURCE).expect("SFC fixture");

    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, SOURCE, path.to_str().expect("UTF-8 fixture path"));
    let [diagnostic] = result.diagnostics.as_slice() else {
        panic!(
            "expected exactly the unchanged identical-type packet\n{}",
            failure_context(&result)
        );
    };
    let start = SOURCE.find("stable as number").expect("control assertion") as u32;
    let end = start + "stable as number".len() as u32;
    assert_eq!(diagnostic.rule_name, RULE);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.message, MESSAGE);
    assert_eq!((diagnostic.start, diagnostic.end), (start, end));
    assert_eq!(diagnostic.help, None);
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    assert_eq!(result.filename.as_str(), path.to_str().expect("UTF-8 path"));
}
