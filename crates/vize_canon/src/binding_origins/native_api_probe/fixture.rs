//! Original public inputs; no generated Vue helpers or mapper output is used.
use serde_json::json;
use std::{fs, path::Path};

pub(super) const PROBE: &str = r#"import { useTemplateRef as actual } from 'vue-authentic';
import * as RealVue from 'vue-authentic';
import { useTemplateRef as other } from 'vue-other';
import { useTemplateRef as trapped } from 'vue';
import { annotated, custom } from './custom';
import type { useTemplateRef as TypeOnly } from 'vue-authentic';
const valueAlias = actual;
/*😀*/ void actual;
void /*probe:namespace*/ RealVue.useTemplateRef;
void /*probe:other*/ other;
void /*probe:trapped*/ trapped;
void /*probe:annotated*/ annotated;
void /*probe:value-alias*/ valueAlias;
void /*probe:ambient-vue*/ GlobalVue;
void /*probe:ambient-shape*/ GlobalShape;
void /*probe:merged*/ GlobalMerged;
void /*probe:class*/ GlobalClass;
void /*probe:enum*/ GlobalEnum;
void /*probe:namespace-global*/ GlobalNamespace;
void /*probe:destructure*/ GlobalDestructure;
void /*probe:type-only*/ TypeOnly;
void /*probe:missing*/ MissingRuntimeRef;
export function shadow(actual: typeof custom) {
  void /*probe:shadow*/ actual;
}
"#;

pub(super) const CASES: [(&str, &str); 15] = [
    ("namespace", "RealVue.useTemplateRef"),
    ("other", "other"),
    ("trapped", "trapped"),
    ("annotated", "annotated"),
    ("value-alias", "valueAlias"),
    ("ambient-vue", "GlobalVue"),
    ("ambient-shape", "GlobalShape"),
    ("merged", "GlobalMerged"),
    ("class", "GlobalClass"),
    ("enum", "GlobalEnum"),
    ("namespace-global", "GlobalNamespace"),
    ("destructure", "GlobalDestructure"),
    ("type-only", "TypeOnly"),
    ("missing", "MissingRuntimeRef"),
    ("shadow", "actual"),
];
pub(super) fn write(root: &Path) {
    fs::write(root.join("probe.ts"), PROBE).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions":{
                "strict":true,"noEmit":true,"skipLibCheck":true,
                "module":"ESNext","moduleResolution":"Bundler","target":"ES2024",
                "moduleDetection":"legacy","types":[],
                "paths":{"vue":["./custom-vue.ts"]}
            },
            "files":["probe.ts","custom.ts","custom-vue.ts","globals.d.ts",
                "merged.d.ts","opaque-globals.ts","partial.ts"]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(root.join("custom.ts"), r#"import { useTemplateRef as actual } from 'vue-authentic';
import type { ShallowRef } from 'vue-authentic';
export function custom<T = unknown, K extends string = string>(_key: K): Readonly<ShallowRef<T | null>> { throw new Error('test-only'); }
export const annotated: typeof actual = custom;
"#).unwrap();
    fs::write(root.join("custom-vue.ts"), r#"export * from 'vue-authentic';
export function useTemplateRef<T = unknown, K extends string = string>(_key: K): Readonly<import('vue-authentic').ShallowRef<T | null>> { throw new Error('test-only'); }
"#).unwrap();
    fs::write(root.join("globals.d.ts"), r#"export {};
declare global {
  const GlobalVue: typeof import('vue-authentic').useTemplateRef;
  const GlobalShape: <T = unknown, K extends string = string>(key: K) => Readonly<import('vue-authentic').ShallowRef<T | null>>;
  function GlobalMerged(key: string): string;
}
"#).unwrap();
    fs::write(
        root.join("merged.d.ts"),
        r#"export {};
declare global {
  function GlobalMerged(key: number): number;
  namespace GlobalMerged { const brand: unique symbol; }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("opaque-globals.ts"),
        r#"class GlobalClass {}
enum GlobalEnum { One }
namespace GlobalNamespace { export const marker = 1; }
const { GlobalDestructure } = { GlobalDestructure: 1 };
"#,
    )
    .unwrap();
    repair(root);
}
pub(super) fn partial(root: &Path) {
    fs::write(root.join("partial.ts"), "export const marker = ;\n").unwrap();
}
pub(super) fn repair(root: &Path) {
    fs::write(root.join("partial.ts"), "export const marker = 1;\n").unwrap();
}
pub(super) fn position(label: &str, token: &str) -> u32 {
    let marker = format!("/*probe:{label}*/ ");
    let base = PROBE.find(&marker).unwrap() + marker.len();
    let extra = if label == "namespace" {
        "RealVue.".len()
    } else {
        0
    };
    assert_eq!(PROBE.get(base..base + token.len()), Some(token));
    u32::try_from(PROBE.get(..base + extra).unwrap().encode_utf16().count()).unwrap()
}

pub(super) fn actual_position() -> u32 {
    let marker = "/*😀*/ void ";
    let offset = PROBE.find(marker).unwrap() + marker.len();
    u32::try_from(PROBE.get(..offset).unwrap().encode_utf16().count()).unwrap()
}
