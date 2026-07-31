//! Generic `$emit` overloads must not turn ordinary `on*` props into listeners.
//!
//! Oracle: TypeScript 6.0.3 and Vue 3.6.0-beta.10.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn generic_emit_guards_are_overload_order_independent() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-emit-overload-order",
        &[
            (
                "src/components.ts",
                r#"type Props = {
  count: number
  onSave?: (value: number) => void
}

export const GenericFirstCallbackChild = null as unknown as {
  new(): {
    $props: Props
    $emit: {
      (event: string, ...args: any[]): void
      (event: 'save', value: string): void
    }
  }
}

export const GenericLastCallbackChild = null as unknown as {
  new(): {
    $props: Props
    $emit: {
      (event: 'save', value: string): void
      (event: string, ...args: any[]): void
    }
  }
}

export const AnyEmitChild = null as unknown as {
  new(): {
    $props: Props
    $emit: any
  }
}

export const TypedOnlyChild = null as unknown as {
  new(): {
    $props: { count: number; onSave?: (value: string) => void }
    $emit: (event: 'save', value: string) => void
  }
}
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import { AnyEmitChild, GenericFirstCallbackChild, GenericLastCallbackChild, TypedOnlyChild } from './components'
const stringHandler = (_value: string) => {}
</script>

<template>
  <GenericFirstCallbackChild :on-save="stringHandler" />
  <GenericLastCallbackChild :on-save="stringHandler" />
  <AnyEmitChild :on-save="stringHandler" />
  <TypedOnlyChild :on-save="stringHandler" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };
    let actual: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| {
            (
                file.as_str(),
                *code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            ("src/Parent.vue", Some(2322), "7:31".to_string()),
            ("src/Parent.vue", Some(2322), "8:30".to_string()),
            ("src/Parent.vue", Some(2322), "9:18".to_string()),
            ("src/Parent.vue", Some(2345), "10:4".to_string()),
        ],
        "generic-first and generic-last overloads must share one result, and unbound required props must remain enforced: {snapshot:#?}"
    );
}
