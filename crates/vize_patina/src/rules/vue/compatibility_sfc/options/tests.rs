use super::NoPropsShadow;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn shadowed(source: &str) -> Vec<&str> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoPropsShadow));
    let mut diagnostics = Linter::with_registry(registry)
        .lint_sfc(source, "Item.vue")
        .diagnostics;
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
        .iter()
        .filter_map(|diagnostic| source.get(diagnostic.start as usize..diagnostic.end as usize))
        .collect()
}

#[test]
fn props_declared_in_the_sibling_script_are_shadowed() {
    let source = r#"<script lang="ts">
import type { CollapsibleRootProps } from '../Collapsible'
interface Base { dir?: 'ltr' | 'rtl' }
export interface ItemProps extends Base, Omit<CollapsibleRootProps, 'open'> {
  disabled?: boolean
  value: string
}
</script>
<script setup lang="ts">
import { computed, toRefs } from 'vue'
const props = withDefaults(defineProps<ItemProps>(), { disabled: false })
const disabled = computed(() => props.disabled)
const { dir, value: current } = toRefs(props)
const open = computed(() => current.value)
</script>
<template><div /></template>"#;
    assert_eq!(shadowed(source), ["disabled", "dir"]);
}

#[test]
fn an_imported_props_type_named_in_the_sibling_script_stays_unresolved() {
    let source = r#"<script lang="ts">
import type { ItemProps } from './types'
export type { ItemProps }
</script>
<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<ItemProps>()
const disabled = computed(() => props.disabled)
</script>
<template><div /></template>"#;
    assert_eq!(shadowed(source), Vec::<&str>::new());
}

#[test]
fn an_annotated_binding_reports_its_type_annotation_too() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'
import type { Ref } from 'vue'
const props = defineProps<{ items?: string[]; modelValue?: number }>()
const items: string[] = Array(10)
const modelValue: Ref<number> = ref(0), other = 1
</script>
<template><div /></template>"#;
    assert_eq!(
        shadowed(source),
        ["items: string[]", "modelValue: Ref<number>"]
    );
}
