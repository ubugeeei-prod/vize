//! Options API public instance members must survive declaration emit (#4010).
//!
//! Canon synthesizes `data` / `computed` / `methods` / setup-return bindings for
//! template checking, but the batch lane rebuilt the emitted default export from
//! `Props`, `Emits` and `Slots` alone. A component therefore type-checked
//! internally while its emitted type said its real public members did not exist.
//!
//! The oracle is `vue-tsc --emitDeclarationOnly` over the same authored sources:
//! [`fixtures::DOWNSTREAM`] compiles clean against vue-tsc's declarations, so it
//! must compile clean — with every negative control still firing — against
//! Vize's.

use std::path::{Path, PathBuf};

use super::super::{
    BatchTypeChecker, DeclarationEmitOptions, create_project_case, relative_path,
    resolve_test_tsgo_binary, with_workspace_node_modules_override,
};
use super::public_instance_contract::{assert_downstream_compiler, resolve_javascript_tsc};
use crate::batch::TypeChecker;

mod fixtures;

const VUE2: &str = r#"<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  data() {
    return { count: 1 };
  },
  computed: {
    doubled(): number {
      return this.count * 2;
    },
  },
  methods: {
    add(step: number): number {
      return this.count + step;
    },
  },
});
</script>

<template><span /></template>
"#;

const VUE2_DOWNSTREAM: &str = r#"import Legacy from "../types/Legacy.vue";

type IsAny<T> = 0 extends 1 & T ? true : false;

declare const legacy: InstanceType<typeof Legacy>;
const legacyCount: number = legacy.count;
const legacyDoubled: number = legacy.doubled;
const legacyAdd: number = legacy.add(1);
const legacyCountIsAny: false = null as unknown as IsAny<typeof legacy.count>;
// @ts-expect-error a Vue 2 data member keeps its exact type
const legacyCountAsString: string = legacy.count;
// @ts-expect-error a Vue 2 method parameter keeps its exact type
const legacyAddWrong: number = legacy.add("1");

void [legacyCount, legacyDoubled, legacyAdd, legacyCountIsAny, legacyCountAsString, legacyAddWrong];
"#;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn test_node_modules() -> PathBuf {
    workspace_root().join("tests").join("node_modules")
}

/// `create_project_case` against the installed Vue runtime: the whole point of
/// the matrix is the member set Vue's own `DefineComponent` exposes, which the
/// facade stub cannot model. Asserted rather than skipped so a missing runtime
/// fails loudly instead of turning the matrix into a no-op.
fn create_case(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let node_modules = test_node_modules();
    assert!(
        node_modules.join("vue/package.json").is_file(),
        "the Options API declaration matrix requires the installed real Vue runtime"
    );
    with_workspace_node_modules_override(
        Some(
            node_modules
                .to_str()
                .expect("test node_modules path should be UTF-8"),
        ),
        || create_project_case(name, files),
    )
}

fn write_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "rootDir": "src",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn checker(project_root: &Path, options_api: bool, legacy_vue2: bool) -> BatchTypeChecker {
    let mut checker = BatchTypeChecker::new(project_root).expect("batch checker construction");
    if legacy_vue2 {
        checker.enable_legacy_vue2();
    } else if options_api {
        checker.enable_options_api();
    }
    checker.scan_project().expect("project scan");
    checker
}

/// The complete diagnostic list — file, code, authored line/column, severity and
/// message — so a lane is asserted by full equality rather than by a substring.
fn project_diagnostics(
    project_root: &Path,
    options_api: bool,
) -> Vec<(String, Option<u32>, u32, u32, u8, String)> {
    let result = checker(project_root, options_api, false)
        .check_project()
        .expect("project check");
    let mut rows: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file).to_string(),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.severity,
                diagnostic.message.to_string(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn emit_declarations(
    project_root: &Path,
    out_dir: &Path,
    options_api: bool,
    legacy_vue2: bool,
) -> Vec<String> {
    let emitted = checker(project_root, options_api, legacy_vue2)
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.to_path_buf()))
        .expect("Options API declarations should emit");
    let mut paths: Vec<_> = emitted
        .files
        .iter()
        .map(|file| relative_path(out_dir, &file.path).to_string())
        .collect();
    paths.sort();
    paths
}

fn write_downstream(project_root: &Path, source: &str) -> PathBuf {
    let downstream_dir = project_root.join("downstream");
    std::fs::create_dir_all(&downstream_dir).unwrap();
    let downstream = downstream_dir.join("verify.tsx");
    std::fs::write(&downstream, source).unwrap();
    downstream
}

#[test]
fn options_api_declarations_expose_the_authored_public_instance() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        return;
    };
    let project_root = create_case("options-api-declaration-instance", fixtures::PROJECT_FILES);
    write_tsconfig(&project_root);

    assert_eq!(
        project_diagnostics(&project_root, true),
        Vec::new(),
        "the source lane (script consumer, template refs, parent refs) must stay clean"
    );

    let out_dir = project_root.join("types");
    let emitted = emit_declarations(&project_root, &out_dir, true, false);
    assert_eq!(
        emitted,
        [
            "Computed.vue.d.ts",
            "Consumer.d.ts",
            "Data.vue.d.ts",
            "Inherited.vue.d.ts",
            "Methods.vue.d.ts",
            "Options.vue.d.ts",
            "Parent.vue.d.ts",
            "PropsEmits.vue.d.ts",
            "SetupReturn.vue.d.ts",
            "__vize_helpers.d.ts",
            "base.d.ts",
            "greeter.d.ts",
        ],
        "declaration emit must cover every authored module"
    );

    let options_dts = std::fs::read_to_string(out_dir.join("Options.vue.d.ts")).unwrap();
    assert!(
        options_dts.contains(
            "type __VizeAuthoredComponent = Awaited<ReturnType<typeof __setup>>[\"__default__\"];"
        ) && options_dts.contains(
            "type __VizeComponentInstance = Omit<__VizeAuthoredInstance, '$props' | '$emit' | '$slots'> & {"
        ) && options_dts.contains("count: number;"),
        "the emitted instance must keep the authored Options API members:\n{options_dts}"
    );

    let downstream = write_downstream(&project_root, fixtures::DOWNSTREAM);
    assert_downstream_compiler(&tsgo, &downstream);
    if let Some(tsc) = resolve_javascript_tsc() {
        assert_downstream_compiler(&tsc, &downstream);
    }
    let _ = std::fs::remove_dir_all(&project_root);
}

/// The `optionsApi` setting decides whether unknown template names resolve on
/// the public instance. It must not decide what consumers see: the emitted
/// declaration is the authored component either way.
#[test]
fn options_api_setting_moves_template_participation_only() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    // Separate project roots: a second emit into the same root would re-scan the
    // first run's output and collide on the shared helper declarations.
    let enabled_root = create_case(
        "options-api-declaration-shape-on",
        &[("src/Shape.vue", fixtures::SHAPE_ONLY)],
    );
    let disabled_root = create_case(
        "options-api-declaration-shape-off",
        &[("src/Shape.vue", fixtures::SHAPE_ONLY)],
    );
    write_tsconfig(&enabled_root);
    write_tsconfig(&disabled_root);
    let enabled_dir = enabled_root.join("types");
    let disabled_dir = disabled_root.join("types");
    assert_eq!(
        emit_declarations(&enabled_root, &enabled_dir, true, false),
        emit_declarations(&disabled_root, &disabled_dir, false, false),
        "the Options API setting must not change which declarations are emitted"
    );
    let enabled = std::fs::read_to_string(enabled_dir.join("Shape.vue.d.ts")).unwrap();
    let disabled = std::fs::read_to_string(disabled_dir.join("Shape.vue.d.ts")).unwrap();
    assert_eq!(
        enabled, disabled,
        "the Options API setting must not change the emitted declaration shape"
    );
    assert!(
        enabled.contains("count: number;")
            && enabled.contains("doubled(): number;")
            && enabled.contains("add(step: number): number;"),
        "the emitted declaration must keep data/computed/methods in both modes:\n{enabled}"
    );
    let _ = std::fs::remove_dir_all(&enabled_root);
    let _ = std::fs::remove_dir_all(&disabled_root);

    let template_root = create_case(
        "options-api-declaration-template",
        &[("src/Options.vue", fixtures::OPTIONS)],
    );
    write_tsconfig(&template_root);
    assert_eq!(
        project_diagnostics(&template_root, true),
        Vec::new(),
        "an enabled Options API resolves the template binding"
    );
    assert_eq!(
        project_diagnostics(&template_root, false),
        [(
            "src/Options.vue".to_string(),
            Some(2304),
            9,
            14,
            1,
            "Cannot find name 'count'.".to_string(),
        )],
        "a disabled Options API leaves the template binding unresolved"
    );
    let _ = std::fs::remove_dir_all(&template_root);
}

/// The Vue 2.7 lane intersects the authored instance without the Vue 3
/// normalization, so it needs its own coverage. No local `vue-tsc` oracle
/// exists for it: the installed Vue is 3.x, so the expectations here come from
/// Vue's own `defineComponent` return type rather than from an upstream run.
///
/// This covers the `defineComponent` shape only. A Vue 2.7 SFC that exports a
/// bare options object is wrapped by the legacy `__vizeDefineComponent`, which
/// returns the options object rather than a constructor, so
/// `__VizeAuthoredInstance` collapses to `{}` and its members stay invisible to
/// consumers — a separate gap that needs Vue 2's `ThisType` instance resolution
/// and is tracked apart from #4010.
#[test]
fn vue2_options_api_declarations_expose_the_authored_public_instance() {
    let Some(tsgo) = resolve_test_tsgo_binary() else {
        return;
    };
    let project_root = create_case("options-api-declaration-vue2", &[("src/Legacy.vue", VUE2)]);
    write_tsconfig(&project_root);

    let out_dir = project_root.join("types");
    assert!(
        emit_declarations(&project_root, &out_dir, true, true)
            .contains(&"Legacy.vue.d.ts".to_string()),
        "the Vue 2.7 lane must emit a declaration for the authored component"
    );
    let legacy_dts = std::fs::read_to_string(out_dir.join("Legacy.vue.d.ts")).unwrap();
    assert!(
        legacy_dts.contains("type __VizeComponentInstance = __VizeAuthoredInstance & {")
            && legacy_dts.contains("count: number;"),
        "the Vue 2.7 instance must keep the authored Options API members:\n{legacy_dts}"
    );

    let downstream = write_downstream(&project_root, VUE2_DOWNSTREAM);
    assert_downstream_compiler(&tsgo, &downstream);
    let _ = std::fs::remove_dir_all(&project_root);
}
