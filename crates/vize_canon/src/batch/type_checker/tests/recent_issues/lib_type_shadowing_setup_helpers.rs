//! Setup-side helpers reach shadowed lib and DOM types through `globalThis`
//! (#12536): macro signatures, template ref registries, CSS modules and
//! component `v-model` payloads.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;
use oxc_syntax::identifier::is_identifier_part;
use std::path::Path;

/// A same-arity `never` poison, and how many references the host's
/// [content mapper, batch] projections generate. The template-ref registry
/// targets name the shared SDK declarations, whose DOM lib references
/// resolve outside every authored scope, so the host spells none of those.
const SETUP_LIB_TYPES: &[(&str, &str, [usize; 2])] = &[
    ("Element", "type Element = never", [0, 0]),
    ("Exclude", "type Exclude<T, U> = never", [1, 1]),
    ("Extract", "type Extract<T, U> = never", [1, 1]),
    (
        "HTMLElementTagNameMap",
        "type HTMLElementTagNameMap = never",
        [0, 0],
    ),
    ("NonNullable", "type NonNullable<T> = never", [0, 1]),
    ("Readonly", "type Readonly<T> = never", [2, 2]),
    (
        "SVGElementTagNameMap",
        "type SVGElementTagNameMap = never",
        [0, 0],
    ),
];
const MAPPER: usize = 0;
const BATCH: usize = 1;

const SETUP_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ items: string[]; modelValue?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
defineExpose({ reset: () => emit('update:modelValue', '') })
</script>

<template>
  <ul><li v-for="item in items" :key="item">{{ item }}</li></ul>
</template>
"#;

#[derive(Clone, Copy, PartialEq)]
enum Poison {
    /// The poison lines under names nothing references.
    None,
    Declared,
    Imported,
}

fn host_source(poison: Poison) -> std::string::String {
    let mut lines = std::string::String::new();
    for (name, declaration, _) in SETUP_LIB_TYPES {
        let line = match poison {
            Poison::None => declaration.replacen(name, &format!("Unshadowed{name}"), 1),
            Poison::Declared => std::string::String::from(*declaration),
            Poison::Imported => format!("import type {{ {name} }} from './setup-lib-types'"),
        };
        lines.push_str(&line);
        lines.push('\n');
    }
    format!(
        r#"<script lang="ts">
{lines}export const setupLibTypePoison = true
</script>

<script setup lang="ts" generic="T extends string">
import {{ ref, useCssModule, useTemplateRef }} from 'vue'
import SetupChild from './SetupChild.vue'

const props = defineProps<{{ label: T; items: string[] }}>()
const button = useTemplateRef('button')
const child = useTemplateRef('child')
const styles = useCssModule()
const chosen = ref('')
void child
</script>

<template>
  <button ref="button" :class="styles.root" @click="button?.focus()">{{{{ props.label }}}}</button>
  <SetupChild ref="child" v-model="chosen" :items="props.items" />
</template>

<style module>
.root {{ color: red; }}
</style>
"#
    )
}

fn setup_lib_types_module() -> std::string::String {
    SETUP_LIB_TYPES
        .iter()
        .map(|(_, declaration, _)| format!("export {declaration}\n"))
        .collect()
}

/// References to `name` that are neither qualified nor part of a longer identifier.
fn unqualified_count(source: &str, name: &str) -> usize {
    source
        .match_indices(name)
        .filter(|&(start, matched)| {
            let before = source
                .get(..start)
                .and_then(|head| head.chars().next_back());
            let after = source
                .get(start + matched.len()..)
                .and_then(|tail| tail.chars().next());
            before.is_none_or(|c| c != '.' && c != '$' && !is_identifier_part(c))
                && after.is_none_or(|c| c != '$' && !is_identifier_part(c))
        })
        .count()
}

/// The poison is the only unqualified spelling, and every generated reference
/// is qualified.
fn assert_only_the_poison_is_unqualified(generated: &str, lane: usize) {
    for (name, _, references) in SETUP_LIB_TYPES {
        let qualified = format!("globalThis.{name}");
        let qualified_count = generated
            .match_indices(qualified.as_str())
            .filter(|&(start, matched)| {
                generated
                    .get(start + matched.len()..)
                    .and_then(|tail| tail.chars().next())
                    .is_none_or(|c| c != '$' && !is_identifier_part(c))
            })
            .count();
        assert_eq!(
            (unqualified_count(generated, name), qualified_count),
            (1, references[lane]),
            "{name}: (unqualified, qualified) references:\n{generated}"
        );
    }
}

#[test]
fn setup_helpers_name_shadowed_lib_types_through_global_this() {
    let options = crate::batch::ContentMapperTransformOptions::default()
        .with_external_component_attrs_module(true)
        .with_external_component_type_helpers(true)
        .with_shared_virtual_module(true);
    for poison in [Poison::Declared, Poison::Imported] {
        let transform = crate::batch::generate_vue_content_mapper_transform_with_options(
            Path::new("/project/src/SetupHost.vue"),
            &host_source(poison),
            options,
        )
        .expect("setup host should project");
        assert_only_the_poison_is_unqualified(&transform.text, MAPPER);
    }
}

#[test]
fn authored_lib_types_cannot_capture_setup_helpers() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let lib_types = setup_lib_types_module();
    let mut snapshots = Vec::new();
    for (case, poison) in [
        ("setup-lib-type-shadowing-clean", Poison::None),
        ("setup-lib-type-shadowing-declared", Poison::Declared),
        ("setup-lib-type-shadowing-imported", Poison::Imported),
    ] {
        let host = host_source(poison);
        let project_root = create_project_case(
            case,
            &[
                ("src/SetupChild.vue", SETUP_CHILD),
                ("src/setup-lib-types.ts", &lib_types),
                ("src/SetupHost.vue", &host),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&project_root);
        if poison != Poison::None {
            let host_path = project_root.join("src/SetupHost.vue");
            let mut project =
                VirtualProject::new(&project_root).expect("virtual project should open");
            project
                .register_paths(&[project_root.join("src/SetupChild.vue"), host_path.clone()])
                .expect("fixture should register");
            let generated = &project
                .find_by_original(&host_path)
                .expect("host virtual file should exist")
                .content;
            assert_only_the_poison_is_unqualified(generated, BATCH);
        }
        let _ = std::fs::remove_dir_all(&project_root);
        let Some(snapshot) = snapshot else {
            return;
        };
        snapshots.push(snapshot);
    }
    assert_eq!(snapshots[0], [], "the clean fixture is valid");
    assert!(
        snapshots.iter().all(|snapshot| *snapshot == snapshots[0]),
        "shadowing a lib type must not change any diagnostic: {snapshots:#?}"
    );
}
