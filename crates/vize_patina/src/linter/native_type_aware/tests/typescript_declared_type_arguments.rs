//! #13002: `type/no-unnecessary-type-arguments` on generics the script does
//! not declare. The type parameters come from the checker's declaration, as
//! typescript-eslint's `getTypeParametersFromNode` reads them, whether the
//! generic is imported, a library member or declared in the script.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter, Severity};
use std::path::Path;

const RULE: &str = "type/no-unnecessary-type-arguments";
const MESSAGE: &str = "This is the default value for this type parameter, so it can be omitted.";

/// The #13002 report's `tsconfig.json`.
const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022",
    "lib": ["ES2022", "DOM"]
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#;

/// Lint `component` beside `modules` in a project of their own, optionally
/// with the workspace's Vue, and return each report as
/// `(line, column, end line, end column)`.
fn reports(
    modules: &[(&str, &str)],
    component: &str,
    vue: Option<&Path>,
) -> Vec<(u32, u32, u32, u32)> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("declared-type-arguments-13002-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let root = project.path();
    std::fs::create_dir(root.join("src")).expect("source directory");
    std::fs::write(root.join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(root.join("package.json"), "{ \"private\": true }\n").expect("package");
    if let Some(vue) = vue {
        std::fs::create_dir(root.join("node_modules")).expect("node_modules");
        #[cfg(unix)]
        std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).expect("Vue link");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(vue, root.join("node_modules/vue")).expect("Vue link");
    }
    for (name, text) in modules {
        std::fs::write(root.join("src").join(name), text).expect("module");
    }
    let path = root.join("src/Args.vue");
    std::fs::write(&path, component).expect("component");
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, component, path.to_str().expect("UTF-8 path"));
    let mut found = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            assert_eq!(diagnostic.message, MESSAGE);
            assert_eq!(diagnostic.severity, Severity::Warning);
            let (line, column) = line_column(component, diagnostic.start);
            let (end_line, end_column) = line_column(component, diagnostic.end);
            (line, column, end_line, end_column)
        })
        .collect::<Vec<_>>();
    found.sort_unstable();
    found
}

/// The #13002 reproduction: an imported interface and function report like
/// the local interface does.
#[test]
fn reported_imported_defaults_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }
    let types = r#"export interface Option<T = unknown> {
  label: string;
  value: T;
}

export function load<T = unknown>(): T[] {
  return [];
}
"#;
    let component = r#"<script setup lang="ts">
import { load, type Option } from "./types";

interface Local<T = unknown> {
  value: T;
}

const imported: Option<unknown>[] = [];
const called = load<unknown>();
const local: Local<unknown>[] = [];
</script>

<template>
  <p>{{ imported.length }} {{ called.length }} {{ local.length }}</p>
</template>
"#;
    assert_eq!(
        reports(&[("types.ts", types)], component, None),
        vec![(8, 24, 8, 31), (9, 21, 9, 28), (10, 20, 10, 27)]
    );
}

/// Classes through `new` and `extends`, interface heritage, overloads,
/// namespace members and DOM methods, with an argument that differs from
/// its default, a parameter without one and an explicit constructor (whose
/// resolved signature owns no type parameters) as controls.
#[test]
fn declarations_outside_the_script_report_their_defaults() {
    if !corsa_available() {
        return;
    }
    let types = r#"export interface Option<T = unknown> {
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
    let component = r#"<script lang="ts">
import * as all from "./types";
import { Explicit, host, type Option, type Required, Store, load, overloaded } from "./types";

export const option: Option<unknown> = { value: undefined };
export const loaded = load<unknown>();
export const store = new Store<string>();
export const overload = overloaded<unknown>();
export const member = host.querySelector<Element>("main");
export const element = document.querySelector<Element>("main");
export const qualified: all.Option<unknown> = { value: undefined };
export class Sub extends Store<string> {}
export interface Extended extends Option<unknown> {}

export const changed: Option<string> = { value: "kept" };
export const required: Required<unknown> = { value: undefined };
export const explicit = new Explicit<string>("kept");
export const specific = document.querySelector<HTMLElement>("main");
</script>
"#;
    assert_eq!(
        reports(&[("types.ts", types)], component, None),
        vec![
            (5, 29, 5, 36),
            (6, 28, 6, 35),
            (7, 32, 7, 38),
            (8, 36, 8, 43),
            (9, 42, 9, 49),
            (10, 47, 10, 54),
            (11, 36, 11, 43),
            (12, 32, 12, 38),
            (13, 42, 13, 49),
        ]
    );
}

/// Vue's own generics, read from the installed package: `T = any` and
/// `T = unknown` defaults report when written out; parameters without a
/// default, a default naming another parameter and the compiler macros stay
/// silent.
#[test]
fn vue_generics_report_from_their_declarations() {
    if !corsa_available() {
        return;
    }
    let Some(vue) = super::macro_positions::workspace_vue() else {
        return;
    };
    let component = r#"<script setup lang="ts">
import { computed, inject, ref, shallowRef, useTemplateRef, type PropType, type Ref } from "vue";

interface Props {
  label?: string;
}
const props = withDefaults(defineProps<Props>(), { label: "x" });
const model = defineModel<string>();
const loose = ref<any>();
const typed: Ref<any> = ref();
const element = useTemplateRef<unknown>("el");
const named = ref<string>("x");
const shallow = shallowRef<number>(1);
const doubled = computed<number>(() => 1);
const paired: Ref<string, string> = ref("x");
const injected = inject<unknown>("key");
const kind = String as PropType<string>;
void [props, model, loose, typed, element, named, shallow, doubled, paired, injected, kind];
</script>
"#;
    assert_eq!(
        reports(&[], component, Some(&vue)),
        vec![(9, 19, 9, 22), (10, 18, 10, 21), (11, 32, 11, 39)]
    );
}

/// `any` written out matches an `any` default, in the script's own
/// declarations as in imported ones. A name the checker cannot resolve types
/// as the error type on both sides; typescript-eslint reports that identity
/// too, but an unresolved name is not taken as proof here.
#[test]
fn written_any_matches_an_any_default_but_unresolved_names_do_not() {
    if !corsa_available() {
        return;
    }
    let component = r#"<script setup lang="ts">
import { Missing } from "./missing";

function read<T = any>(): T | undefined {
  return undefined;
}
interface Loose<T = any> {
  value: T;
}
function unresolved<T = Missing<string>>() {}
const value = read<any>();
const loose: Loose<any> = { value: 1 };
const kept: Loose<unknown> = { value: 1 };
unresolved<Missing<string>>();
void [value, loose, kept];
</script>
"#;
    assert_eq!(
        reports(&[], component, None),
        vec![(11, 20, 11, 23), (12, 20, 12, 23)]
    );
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, character) in source.char_indices() {
        if index >= offset as usize {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
