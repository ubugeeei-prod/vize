//! Generated template-scope names cannot capture authored bindings.
//!
//! Per-usage contract aliases are spelled `__V<Role><N>` (see
//! `virtual_ts::generated_names`). The host authors names beside that space:
//! the legacy `__<Component>_Props_<N>` spelling, the bare `__VProps` role,
//! Volar's `__VLS_*`, Vue's `__VUE_*__` and `__V` itself. A template type
//! assertion reads each one; the legacy spelling used to be redeclared by the
//! template scope, which turned `title: number` back into the child's
//! `title: string` and lost the error below.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ title: string; count?: number }>()
</script>

<template>
  <h1>{{ title }} {{ count }}</h1>
</template>
"#;

const HOST: &str = r#"<script setup lang="ts">
import Child from './Child.vue'

type __Child_Props_0 = { title: number }
type __VProps = { title: string }
const __VLS_ctx = { label: 'ctx' }
const __VUE_OPTIONS_API__ = 'options'
const __VComp = 'comp'
const __V = 1
const probe: unknown = { title: 'probe' }
</script>

<template>
  <Child :title="__VLS_ctx.label + __VComp + __VUE_OPTIONS_API__" :count="__V" />
  <Child :title="(probe as __Child_Props_0).title" />
  <Child :title="(probe as __VProps).title" />
</template>
"#;

#[test]
fn authored_names_beside_the_reserved_prefix_keep_their_meaning() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generated-name-hygiene",
        &[("src/Child.vue", CHILD), ("src/Host.vue", HOST)],
    );
    let host_path = project_root.join("src/Host.vue");
    let mut project = VirtualProject::new(&project_root).expect("virtual project should open");
    project
        .register_paths(std::slice::from_ref(&host_path))
        .expect("fixture should register");
    let generated = project
        .find_by_original(&host_path)
        .expect("host virtual file should exist")
        .content
        .clone();
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    // Every usage declares its own compact contract; no authored spelling is
    // declared a second time.
    for usage in 0..3 {
        assert_eq!(
            generated
                .matches(&format!("type __VProps{usage} = "))
                .count(),
            1,
            "usage {usage} declares its contract once:\n{generated}"
        );
    }
    for authored in [
        "type __Child_Props_0 =",
        "type __VProps =",
        "const __VComp =",
    ] {
        assert_eq!(
            generated.matches(authored).count(),
            1,
            "`{authored}` is authored once and never generated:\n{generated}"
        );
    }

    let Some(snapshot) = snapshot else {
        return;
    };
    assert_eq!(
        snapshot,
        [(
            "src/Host.vue".into(),
            Some(2322),
            "15:11:error Type 'number' is not assignable to type 'string'.".into(),
        )],
        "only the authored `title: number` is wrong"
    );
}
