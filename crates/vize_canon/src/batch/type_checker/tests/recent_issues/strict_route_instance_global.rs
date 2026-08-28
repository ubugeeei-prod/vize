//! #4963: the strict template context dropped `$route` while keeping every
//! other `$`-prefixed public-instance member, so a template reading
//! `$route.params.x` reported a false `TS2551` (`Did you mean '$router'?`)
//! even though vue-router augments `ComponentCustomProperties` with both
//! members and `vue-tsc` accepts the same input with zero errors.

use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use crate::virtual_ts::VirtualTsOptions;
use vize_s0::{String, cstr};

/// vue-router's shipped module augmentation registers both `$route` and
/// `$router`; the strict context must expose them the way it exposes any
/// other augmented instance global.
const ROUTER_AUGMENTATION: &str = "import 'vue';\n\
declare module 'vue' {\n\
  interface ComponentCustomProperties {\n\
    $route: { params: Record<string, string>; query: Record<string, string> };\n\
    $router: { push(to: string): void };\n\
  }\n\
}\n";

#[test]
fn strict_template_context_resolves_route_alongside_router() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "strict-route-instance-global",
        &[
            ("src/router.d.ts", ROUTER_AUGMENTATION),
            (
                "src/App.vue",
                r#"<script setup lang="ts"></script>

<template>
  <span>{{ $route.params.id }}</span>
  <button @click="$router.push($route.query.tab)">go</button>
  <i>{{ $undeclaredPlugin.label }}</i>
</template>
"#,
            ),
        ],
    );

    let diagnostics = strict_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    let route_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|(_, _, message)| message.contains("$route") || message.contains("$router"))
        .collect();
    assert!(
        route_diagnostics.is_empty(),
        "augmented $route/$router must resolve on the strict template context: {route_diagnostics:#?}"
    );

    // The strict form still reports a global nothing declares, so the fix
    // cannot have widened the context back to a permissive `any` surface.
    assert_eq!(
        diagnostics
            .iter()
            .filter(|(file, code, message)| {
                file.as_str() == "src/App.vue"
                    && *code == Some(2339)
                    && message.contains("$undeclaredPlugin")
            })
            .count(),
        1,
        "an undeclared strict template global must keep its TS2339: {diagnostics:#?}"
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
}

/// `snapshot_project_diagnostics` with the strict instance-global form the
/// Nuxt detection enables for projects that publish generated types.
///
/// The caller already gated on a resolvable checker binary, so every failure
/// past that point is a real regression and panics instead of skipping.
fn strict_project_diagnostics(project_root: &Path) -> Vec<(String, Option<u32>, String)> {
    let options = BatchTypeCheckerOptions {
        virtual_ts_options: VirtualTsOptions {
            strict_instance_globals: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut checker = BatchTypeChecker::with_options(project_root, options)
        .expect("batch checker should initialize");
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}
