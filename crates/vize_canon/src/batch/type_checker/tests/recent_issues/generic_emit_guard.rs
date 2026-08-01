//! Generic `$emit` overloads must not turn ordinary `on*` props into listeners.
//!
//! Oracle: TypeScript 6.0.3 and Vue 3.6.0-beta.10.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

/// The whole-props failure every usage here produces: it binds the optional
/// `onSave` and never binds the required `count`.
fn missing_count(line: u32, props: &str) -> String {
    cstr!(
        "{line}:4:error Argument of type '{{ onSave: (_value: string) => void; }}' is not assignable to parameter of type '{props} & Record<string, unknown>'.\nProperty 'count' is missing in type '{{ onSave: (_value: string) => void; }}' but required in type '{props}'."
    )
}

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
  onSave?: (value: string) => void
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

    // `TypedOnlyChild` spells its props inline, so TypeScript prints the type
    // rather than the `Props` alias the other three share. Every usage binds a
    // declared `on*` prop and none of them binds `count`: before #3569 a named
    // binding switched the whole-props target to one where every unbound key was
    // optional, and all four were silent.
    let inline_props = "{ count: number; onSave?: ((value: string) => void) | undefined; }";
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2345),
                missing_count(10, inline_props),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                missing_count(7, "Props"),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                missing_count(8, "Props"),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                missing_count(9, "Props"),
            ),
        ],
        "generic-first and generic-last overloads must share one result, and every usage reports the required `count` it never binds"
    );
}
