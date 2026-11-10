//! Slot and template ref helpers reach shadowed lib types through
//! `globalThis` (#12536).
//!
//! The fixture has slot outlets, scoped and inferred slots and a narrowed
//! maybe-ref, but no component listeners, whose relations belong to the
//! listener chain.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;
use oxc_syntax::identifier::is_identifier_part;
use std::path::Path;

/// A same-arity `never` poison, and how many references the host's
/// [content mapper, batch] projections generate.
const SLOT_LIB_TYPES: &[(&str, &str, [usize; 2])] = &[
    ("Exclude", "type Exclude<T, U> = never", [2, 2]),
    ("Extract", "type Extract<T, U> = never", [2, 2]),
    ("NonNullable", "type NonNullable<T> = never", [6, 6]),
    ("PropertyKey", "type PropertyKey = never", [6, 6]),
];
const MAPPER: usize = 0;
const BATCH: usize = 1;

const SLOT_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ items: string[] }>()
defineSlots<{ default(props: { item: string; index: number }): any }>()
</script>

<template>
  <ul><li v-for="(item, index) in items" :key="item"><slot :item="item" :index="index" /></li></ul>
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
    for (name, declaration, _) in SLOT_LIB_TYPES {
        let line = match poison {
            Poison::None => declaration.replacen(name, &format!("Unshadowed{name}"), 1),
            Poison::Declared => std::string::String::from(*declaration),
            Poison::Imported => format!("import type {{ {name} }} from './slot-lib-types'"),
        };
        lines.push_str(&line);
        lines.push('\n');
    }
    format!(
        r#"<script lang="ts">
{lines}export const slotLibTypePoison = true
</script>

<script setup lang="ts">
import {{ inject, ref, type Ref }} from 'vue'
import SlotChild from './SlotChild.vue'

const count = ref(0)
const injected = inject<Ref<number>>('count')
const slotName = ref('extra')
const rows = ['a', 'b']
</script>

<template>
  <SlotChild :items="rows">
    <template #default="{{ item, index }}">{{{{ item.toUpperCase() }}}}<slot name="inner" :item="item" :index="index" /></template>
  </SlotChild>
  <div v-for="row in rows" :key="row"><slot name="row" :row="row" /></div>
  <slot name="extra" v-bind="{{ count }}" />
  <slot :name="slotName" v-bind="{{ count }}" />
  <span v-if="injected">{{{{ injected.toFixed() }}}}</span>
</template>
"#
    )
}

fn slot_lib_types_module() -> std::string::String {
    SLOT_LIB_TYPES
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
    for (name, _, references) in SLOT_LIB_TYPES {
        let qualified = format!("globalThis.{name}");
        assert_eq!(
            (
                unqualified_count(generated, name),
                generated.matches(qualified.as_str()).count()
            ),
            (1, references[lane]),
            "{name}: (unqualified, qualified) references:\n{generated}"
        );
    }
}

#[test]
fn template_helpers_name_shadowed_lib_types_through_global_this() {
    let options = crate::batch::ContentMapperTransformOptions::default()
        .with_external_component_attrs_module(true)
        .with_external_component_type_helpers(true)
        .with_shared_virtual_module(true);
    for poison in [Poison::Declared, Poison::Imported] {
        let transform = crate::batch::generate_vue_content_mapper_transform_with_options(
            Path::new("/project/src/SlotHost.vue"),
            &host_source(poison),
            options,
        )
        .expect("slot host should project");
        assert_only_the_poison_is_unqualified(&transform.text, MAPPER);
    }
}

#[test]
fn authored_lib_types_cannot_capture_template_helpers() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let lib_types = slot_lib_types_module();
    let mut snapshots = Vec::new();
    for (case, poison) in [
        ("slot-lib-type-shadowing-clean", Poison::None),
        ("slot-lib-type-shadowing-declared", Poison::Declared),
        ("slot-lib-type-shadowing-imported", Poison::Imported),
    ] {
        let host = host_source(poison);
        let project_root = create_project_case(
            case,
            &[
                ("src/SlotChild.vue", SLOT_CHILD),
                ("src/slot-lib-types.ts", &lib_types),
                ("src/SlotHost.vue", &host),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&project_root);
        if poison != Poison::None {
            let host_path = project_root.join("src/SlotHost.vue");
            let mut project =
                VirtualProject::new(&project_root).expect("virtual project should open");
            project
                .register_paths(&[project_root.join("src/SlotChild.vue"), host_path.clone()])
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
