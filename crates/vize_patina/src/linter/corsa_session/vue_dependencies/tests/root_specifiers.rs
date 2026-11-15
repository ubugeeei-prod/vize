use super::super::{PathAliases, prepare};

#[test]
fn root_component_rewrite_preserves_non_vue_module_specifiers() {
    let project = tempfile::tempdir().unwrap();
    let project_root = project.path().canonicalize().unwrap();
    let child = project_root.join("Child.vue");
    std::fs::write(
        &child,
        "<script setup lang=\"ts\">defineProps<{ label?: string }>()</script><template />",
    )
    .unwrap();
    std::fs::write(
        project_root.join("api.ts"),
        "export const LABEL = 'chip'; export function describe(size: number): string { return `${LABEL} ${size}`; }",
    )
    .unwrap();
    let source = r#"import Child from "./Child.vue";
import { LABEL } from "./api";
type Api = typeof import("./api");
async function label(): Promise<string> {
  const api = await import("./api");
  return api.describe(api.LABEL.length) + LABEL;
}
const required = require("./api") as Api;
void Child;
void label;
void required;
"#;
    let session = project_root.join("private-session");
    let filename = project_root.join("Lazy.vue");
    let mut cache = vize_l0::FxHashMap::default();
    let prepared = prepare(
        source,
        &filename.to_string_lossy(),
        &session,
        &project_root,
        &PathAliases::default(),
        &mut cache,
        &vize_l0::FxHashSet::default(),
    );

    assert_eq!(prepared.files.len(), 1, "the SFC keeps its private mirror");
    let target = prepared.files.first().unwrap().0.to_string_lossy();
    let rewritten = prepared.code.expect("component import rewrite");
    let expected = format!(
        r#"import Child from "{}";
import {{ LABEL }} from "./api";
type Api = typeof import("./api");
async function label(): Promise<string> {{
  const api = await import("./api");
  return api.describe(api.LABEL.length) + LABEL;
}}
const required = require("./api") as Api;
void Child;
void label;
void required;
"#,
        target.replace('\\', "/")
    );
    assert_eq!(rewritten, expected);
}
