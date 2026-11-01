//! Synthetic Options API controls using the locked, real Vue type owner.
//! They do not reconstruct a reporter's application or exercise Vue 2.

use super::super::{BatchTypeChecker, relative_path, resolve_test_tsgo_binary};
use super::real_vue::create_project_case;
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use vize_l0::cstr;

const AUGMENTATION: &str = r#"import 'vue';
declare module 'vue' {
  interface ComponentCustomProperties {
    $service: { ping(value: string): number };
    $opaque: unknown;
    $dynamic: any;
  }
}
"#;

const COMPONENT: &str = r#"<script lang="ts">
import { defineComponent } from 'vue';
export default defineComponent({
  methods: {
    valid(): number { return this.$service.ping('ok'); },
    invalid(): number { return this.$service.ping(123); },
    missing(): void { this.$unregistered(); },
    opaque(): void { this.$opaque.member; },
    dynamic(): void { this.$dynamic.deep.call(); },
  },
});
</script>
<template><span /></template>
"#;

#[test]
fn authored_options_methods_keep_module_augmentations_typed() {
    let Some(native) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required Options API augmentation controls cannot skip the native runtime"
        );
        return;
    };
    // The external declaration and an explicitly imported source module must
    // both augment the same Vue authority, without confusing their scope.
    for (name, augmentation_path, component) in [
        (
            "external-declaration",
            "src/plugin.d.ts",
            COMPONENT.to_owned(),
        ),
        (
            "imported-source",
            "src/plugin.ts",
            COMPONENT.replace(
                "import { defineComponent } from 'vue';",
                "import { defineComponent } from 'vue'; import './plugin';",
            ),
        ),
    ] {
        let project_root = create_project_case(
            &cstr!("options-api-global-augmentation-{name}"),
            &[
                (augmentation_path, AUGMENTATION),
                ("src/App.vue", &component),
            ],
        );
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            &project_root,
            BatchTypeCheckerOptions::default(),
            Some(&native),
        )
        .expect("augmentation checker should initialize");
        checker
            .scan_project()
            .expect("augmentation project should scan");
        let result = checker
            .check_project()
            .expect("augmentation project should check");
        let projected = checker.virtual_files();
        let virtual_file = projected
            .iter()
            .find(|file| file.original_path.ends_with("App.vue"))
            .expect("actual App.vue projection");
        let generated = std::fs::read_to_string(&virtual_file.virtual_path)
            .expect("materialized production projection");
        assert_eq!(generated, virtual_file.content.as_str());
        let (_, config) = checker.native_project_paths_for_test();
        let whole_diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                serde_json::json!({
                    "file": relative_path(&project_root, &diagnostic.file),
                    "line": diagnostic.line,
                    "column": diagnostic.column,
                    "message": diagnostic.message,
                    "code": diagnostic.code,
                    "severity": diagnostic.severity,
                    "blockType": diagnostic.block_type.as_ref().map(|block| cstr!("{block:?}")),
                })
            })
            .collect();
        let vue_manifest_path = project_root.join("node_modules/vue/package.json");
        let vue_manifest = std::fs::read_to_string(&vue_manifest_path)
            .expect("the real Vue package manifest must exist");
        let packet = serde_json::json!({
            "scope": "synthetic Options API controls with the locked real Vue owner; not the reported app or Vue 2",
            "native": native,
            "vuePackagePath": std::fs::canonicalize(&vue_manifest_path).expect("actual Vue package path"),
            "wholeVuePackageManifest": vue_manifest,
            "augmentationPath": augmentation_path,
            "wholeAugmentation": AUGMENTATION,
            "wholeAuthoredSource": component,
            "virtualPath": virtual_file.virtual_path,
            "wholeGeneratedSource": generated,
            "configPath": config,
            "wholeConfig": std::fs::read_to_string(&config).expect("materialized config"),
            "wholeDiagnostics": whole_diagnostics,
        });
        retain_packet(name, &packet);
        let mut actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    relative_path(&project_root, &diagnostic.file),
                    diagnostic.line + 1,
                    diagnostic.code,
                )
            })
            .collect();
        actual.sort();
        assert_eq!(
            actual,
            [
                ("src/App.vue".into(), 6, Some(2345)),
                ("src/App.vue".into(), 7, Some(2339)),
                ("src/App.vue".into(), 8, Some(2571)),
            ],
            "complete augmentation packet {name}: {packet:#}"
        );
        drop(checker);
        let _ = std::fs::remove_dir_all(project_root);
    }
}

fn retain_packet(name: &str, packet: &serde_json::Value) {
    eprintln!("Options API global augmentation packet {name}={packet}");
    if let Some(directory) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(directory).join("options-api-global-augmentation");
        std::fs::create_dir_all(&directory).expect("augmentation packet directory");
        std::fs::write(
            directory.join(cstr!("{name}.json").as_str()),
            serde_json::to_vec_pretty(packet).expect("whole augmentation packet"),
        )
        .expect("retain whole augmentation packet");
    }
}
