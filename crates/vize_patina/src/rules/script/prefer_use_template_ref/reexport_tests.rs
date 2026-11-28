//! `ref` / `shallowRef` imported from a module other than Vue's.

use std::path::Path;

use crate::linter::Linter;

const RULE: &str = "script/prefer-use-template-ref";

/// The `(rule, start)` of every finding for `sfc` linted as `dir/name`.
fn findings(dir: &Path, name: &str, sfc: &str) -> Vec<(&'static str, u32)> {
    let filename = dir.join(name);
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, filename.to_str().expect("utf-8 temp path"))
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start))
        .collect()
}

fn at(sfc: &str, call: &str) -> Vec<(&'static str, u32)> {
    vec![(RULE, sfc.find(call).expect("call") as u32)]
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    for (path, source) in files {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, source).expect("write");
    }
    dir
}

#[test]
fn follows_a_tsconfig_alias_to_a_module_that_re_exports_its_vue_import() {
    let dir = project(&[
        (
            "tsconfig.json",
            include_str!(
                "../../../../../../tests/_fixtures/differential/linter/prefer-use-template-ref-identity/tsconfig.json.txt"
            ),
        ),
        (
            "common/vueTools.ts",
            include_str!(
                "../../../../../../tests/_fixtures/differential/linter/prefer-use-template-ref-identity/vueTools.ts.txt"
            ),
        ),
    ]);
    let sfc = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/prefer-use-template-ref-identity/AliasedReexport.vue.txt"
    );
    assert_eq!(
        findings(dir.path(), "List.vue", sfc),
        at(sfc, "ref<HTMLElement")
    );
}

#[test]
fn follows_named_star_and_namespace_re_export_chains() {
    let dir = project(&[
        (
            "utils/index.ts",
            "export * from './vue'\nexport * from './math'\n",
        ),
        (
            "utils/math.ts",
            "export const sum = (a: number, b: number) => a + b\n",
        ),
        (
            "utils/vue.ts",
            "export { ref as makeRef, shallowRef } from 'vue'\n",
        ),
    ]);
    let sfc = "<script setup lang=\"ts\">\nimport { makeRef } from './utils'\nimport * as tools from './utils/vue'\nconst root = makeRef(null)\nconst panel = tools.shallowRef(null)\n</script>\n<template><div ref=\"root\"><div ref=\"panel\" /></div></template>\n";
    let root = sfc.find("makeRef(null)").expect("root") as u32;
    let panel = sfc.find("tools.shallowRef(null)").expect("panel") as u32;
    assert_eq!(
        findings(dir.path(), "Chain.vue", sfc),
        vec![(RULE, root), (RULE, panel)]
    );
}

#[test]
fn the_run_scoped_view_resolves_the_same_chain() {
    let dir = project(&[("vue-tools.ts", "export * from 'vue'\n")]);
    let sfc = "<script setup lang=\"ts\">\nimport { ref } from './vue-tools'\nconst root = ref(null)\n</script>\n<template><div ref=\"root\" /></template>\n";
    let filename = dir.path().join("Run.vue");
    let result = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_run_scoped_type_sources()
        .lint_sfc(sfc, filename.to_str().expect("utf-8 temp path"));
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_name, diagnostic.start))
            .collect::<Vec<_>>(),
        at(sfc, "ref(null)")
    );
}

#[test]
fn packages_that_re_export_vue_runtime_provide_its_factories() {
    let dir = project(&[]);
    let termui = include_str!(
        "../../../../../../tests/_fixtures/differential/linter/prefer-use-template-ref-identity/PackageReexport.vue.txt"
    );
    assert_eq!(
        findings(dir.path(), "Termui.vue", termui),
        at(termui, "shallowRef(null)")
    );
    let sfc = "<script setup lang=\"ts\">\nimport { ref } from '@vue/runtime-core'\nconst box = ref(null)\n</script>\n<template><Box ref=\"box\" /></template>\n";
    assert_eq!(findings(dir.path(), "Box.vue", sfc), at(sfc, "ref(null)"));
}

#[test]
fn a_module_that_defines_or_cannot_prove_the_factory_is_not_vue() {
    let dir = project(&[
        (
            "own.ts",
            "export function ref<T>(value: T) { return { value } }\n",
        ),
        (
            "local.ts",
            "function shallowRef() {}\nexport { shallowRef }\n",
        ),
        ("a.ts", "export const ref = () => null\n"),
        ("ambiguous.ts", "export * from './a'\nexport * from 'vue'\n"),
        ("unresolved.ts", "export * from './missing'\n"),
        (
            "shadowed.ts",
            "export * from 'vue'\nexport const ref = () => null\n",
        ),
    ]);
    for import in [
        "import { ref } from './own'",
        "import { shallowRef as ref } from './local'",
        "import { ref } from './ambiguous'",
        "import { ref } from './unresolved'",
        "import { ref } from './shadowed'",
        "import { ref } from './missing'",
        "import { ref } from 'not-installed'",
        "import ref from './own'",
    ] {
        let sfc = format!(
            "<script setup lang=\"ts\">\n{import}\nconst root = ref(null)\n</script>\n<template><div ref=\"root\" /></template>\n"
        );
        assert_eq!(findings(dir.path(), "Own.vue", &sfc), vec![], "{import}");
    }
}
