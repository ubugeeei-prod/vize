
const IMPORTED_DEFAULTS_RULE: &str = "type/no-unnecessary-type-arguments";
const IMPORTED_DEFAULTS_MESSAGE: &str =
    "This is the default value for this type parameter, so it can be omitted.";
const IMPORTED_DEFAULTS_TSCONFIG: &str = r#"{
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
const IMPORTED_DEFAULTS_TYPES: &str = r#"export interface Option<T = unknown> {
  value: T;
}
export function load<T = unknown>(): T | undefined {
  return undefined;
}
export class Store<T = string> {
  value?: T;
}
export class Explicit<T = string> {
  constructor(public value: T) {}
}
export interface Required<T> {
  value: T;
}
export function overloaded<T = unknown>(value?: T): T | undefined;
export function overloaded(value?: unknown): unknown {
  return value;
}
export interface QueryHost {
  querySelector<K extends keyof HTMLElementTagNameMap>(selectors: K): HTMLElementTagNameMap[K] | null;
  querySelector<E extends Element = Element>(selectors: string): E | null;
}
export declare const host: QueryHost;
"#;
const IMPORTED_DEFAULTS_SOURCE: &str = r#"<script setup lang="ts">
import { Explicit, host, type Option, type Required, Store, load, overloaded } from "./types";

type Local<T = unknown> = T;

const importedOption: Option<unknown> = { value: undefined };
const importedLoad = load<unknown>();
const importedStore = new Store<string>();
const importedOverload = overloaded<unknown>();
const importedMethod = host.querySelector<Element>("main");
const localValue: Local<unknown> = undefined;
const element = document.querySelector<Element>("main");

const changed: Option<string> = { value: "kept" };
const required: Required<unknown> = { value: undefined };
const explicit = new Explicit<string>("kept");
const specific = document.querySelector<HTMLElement>("main");
</script>

<template>
  <p>{{ importedOption.value }} {{ importedLoad }} {{ importedStore.value }} {{ importedOverload }} {{ importedMethod }} {{ localValue }} {{ element }} {{ changed.value }} {{ required.value }} {{ explicit.value }} {{ specific }}</p>
</template>
"#;

#[test]
fn imported_and_library_defaults_match_local_complete_packets() {
    if !corsa_available() {
        return;
    }

    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("no-unnecessary-type-arguments-imported-13002-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(
        project.path().join("tsconfig.json"),
        IMPORTED_DEFAULTS_TSCONFIG,
    )
    .expect("tsconfig");
    std::fs::write(source_dir.join("types.ts"), IMPORTED_DEFAULTS_TYPES)
        .expect("imported declarations");
    let path = source_dir.join("Args.vue");
    std::fs::write(&path, IMPORTED_DEFAULTS_SOURCE).expect("SFC fixture");

    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![IMPORTED_DEFAULTS_RULE.into()]));
    let result = lint_sfc_with_corsa(
        &linter,
        IMPORTED_DEFAULTS_SOURCE,
        path.to_str().expect("UTF-8 fixture path"),
    );
    let expected = [
        "Option<unknown>",
        "load<unknown>",
        "Store<string>",
        "overloaded<unknown>",
        "querySelector<Element>",
        "Local<unknown>",
        "document.querySelector<Element>",
    ]
    .map(|needle| {
        let argument = needle.find('<').expect("type argument") + 1;
        let start = IMPORTED_DEFAULTS_SOURCE.find(needle).expect("reported use") + argument;
        let end = start + needle.len() - argument - 1;
        (start as u32, end as u32)
    });
    let mut diagnostics = result.diagnostics.iter().collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    assert_eq!(diagnostics.len(), expected.len(), "{:#?}", result.diagnostics);
    for (diagnostic, (start, end)) in diagnostics.into_iter().zip(expected) {
        assert_eq!(diagnostic.rule_name, IMPORTED_DEFAULTS_RULE);
        assert_eq!(diagnostic.severity, crate::Severity::Warning);
        assert_eq!(diagnostic.message, IMPORTED_DEFAULTS_MESSAGE);
        assert_eq!((diagnostic.start, diagnostic.end), (start, end));
        assert_eq!(diagnostic.help, None);
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
    assert_eq!((result.error_count, result.warning_count), (0, 7));
    assert_eq!(result.filename.as_str(), path.to_str().expect("UTF-8 path"));
}
