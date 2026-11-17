//! #13086, #13085 and #12992: `type/no-unnecessary-type-assertion` decided on
//! checker facts (receiver types, type-parameter constraints, enum members).
//! The issue-authored files are the pinned differential inputs.
use super::super::{corsa_available, failure_context, lint_sfc_with_corsa};
use crate::{LintPreset, LintResult, Linter};
use std::path::Path;

mod upstream_12992;

const RULE: &str = "type/no-unnecessary-type-assertion";
const UNCHANGED: &str =
    "This assertion is unnecessary since it does not change the type of the expression.";
const CONTEXTUAL: &str =
    "This assertion is unnecessary since the receiver accepts the original type of the expression.";

macro_rules! pinned {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/no-unnecessary-type-assertion-13085-13086/",
            $path
        ))
    };
}

const TSCONFIG: &str = pinned!("tsconfig.json");
const HELPERS: &str = pinned!("src/helpers.ts.txt");
const ANY_NON_NULL: &str = pinned!("src/AnyNonNull.vue.txt");
const GENERIC_DEFAULT: &str = pinned!("src/NonNull.vue.txt");
const ARROW: &str = pinned!("src/Arrow.vue.txt");

/// Declarations whose missing annotation leaves no receiver type, beside the
/// annotated ones that have one.
const ANY_RECEIVERS: &str = r#"<script setup lang="ts">
declare const payload: any;

class Holder {
  field = payload!;
  annotated: string = payload!;
}
const withDefault = (value = payload!) => value;
const annotatedDefault = (value: string = payload!) => value;
</script>

<template>
  <p>{{ Holder }} {{ withDefault }} {{ annotatedDefault }}</p>
</template>
"#;

const GENERIC_BARE: &str = r#"<script setup lang="ts" generic="T">
import { computed } from "vue";

import { takesObject } from "./helpers";

const { value } = defineProps<{ value: T }>();

const label = computed(() => takesObject(value!));
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;

/// A type parameter's constraint decides whether it may be nullish.
const CONSTRAINTS: &str = r#"<script setup lang="ts">
import { takesObject } from "./helpers";

const mixed = <T,>(value: T | string) => takesObject(value!);
const nullable = <T extends string | null>(value: T) => takesObject(value!);
const solid = <T extends string>(value: T) => takesObject(value!);
</script>

<template>
  <p>{{ mixed(1) }} {{ nullable("a") }} {{ solid("a") }}</p>
</template>
"#;

struct Project {
    _dir: tempfile::TempDir,
    source: std::path::PathBuf,
}

fn project(prefix: &str, tsconfig: &str, files: &[(&str, &str)]) -> Project {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let dir = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(workspace)
        .expect("temporary project");
    let source = dir.path().join("src");
    std::fs::create_dir(&source).expect("source directory");
    std::fs::write(dir.path().join("tsconfig.json"), tsconfig).expect("tsconfig");
    for (name, text) in files {
        std::fs::write(source.join(name), text).expect("project file");
    }
    Project { _dir: dir, source }
}

fn lint(project: &Project, name: &str, source: &str) -> LintResult {
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let path = project.source.join(name);
    std::fs::write(&path, source).expect("SFC fixture");
    lint_sfc_with_corsa(&linter, source, path.to_str().expect("UTF-8 fixture path"))
}

/// `(start, end, message)` of every report of the rule.
fn reports(result: &LintResult) -> Vec<(u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            (
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

/// The `!` that ends the first occurrence of `needle`.
fn bang(source: &str, needle: &str) -> (u32, u32) {
    let start = source.find(needle).expect("needle in fixture") + needle.len() - 1;
    (start as u32, start as u32 + 1)
}

#[test]
fn any_operand_needs_an_annotated_receiver_13086() {
    if !corsa_available() {
        return;
    }
    let project = project("no-unnecessary-assertion-any-13086-", TSCONFIG, &[]);
    let result = lint(&project, "AnyNonNull.vue", ANY_NON_NULL);
    let typed = bang(ANY_NON_NULL, "const typed: string = payload!");
    let passed = bang(ANY_NON_NULL, "takesObject(payload!");
    assert_eq!(
        reports(&result),
        vec![
            (typed.0, typed.1, CONTEXTUAL),
            (passed.0, passed.1, CONTEXTUAL)
        ],
        "{}",
        failure_context(&result)
    );

    let result = lint(&project, "Receivers.vue", ANY_RECEIVERS);
    let annotated = bang(ANY_RECEIVERS, "annotated: string = payload!");
    let annotated_default = bang(ANY_RECEIVERS, "(value: string = payload!");
    assert_eq!(
        reports(&result),
        vec![
            (annotated.0, annotated.1, CONTEXTUAL),
            (annotated_default.0, annotated_default.1, CONTEXTUAL),
        ],
        "{}",
        failure_context(&result)
    );
}

#[test]
fn unconstrained_type_parameter_keeps_its_non_null_assertion_13085() {
    if !corsa_available() {
        return;
    }
    let project = project(
        "no-unnecessary-assertion-type-parameter-13085-",
        TSCONFIG,
        &[("helpers.ts", HELPERS)],
    );
    // Without the `!`, every one of these calls is TS2345: `T` may be `null`
    // or `undefined`. typescript-eslint reports the arrow; vize does not.
    for (name, source) in [
        ("NonNull.vue", GENERIC_DEFAULT),
        ("Bare.vue", GENERIC_BARE),
        ("Arrow.vue", ARROW),
    ] {
        let result = lint(&project, name, source);
        assert_eq!(reports(&result), vec![], "{}", failure_context(&result));
    }

    // Only the `string`-constrained parameter makes `value!` a no-op.
    let result = lint(&project, "Constraints.vue", CONSTRAINTS);
    let solid = bang(
        CONSTRAINTS,
        "<T extends string>(value: T) => takesObject(value!",
    );
    assert_eq!(
        reports(&result),
        vec![(solid.0, solid.1, UNCHANGED)],
        "{}",
        failure_context(&result)
    );
}
