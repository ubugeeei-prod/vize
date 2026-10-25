use super::*;
use vize_l0::String;

fn names(source: &str) -> Vec<String> {
    collect_template_asset_urls(source, None, Some("App.vue"))
        .into_iter()
        .map(|asset| asset.var_name)
        .collect()
}

#[test]
fn reserves_module_bindings_in_each_real_script_grammar() {
    let scripts = [
        r#"<script>const _imports_0 = 'authored'; export default { value: _imports_0 }</script>"#,
        r#"<script>if (true) { var _imports_0 = 'authored' } export default {}</script>"#,
        r#"<script setup>const _imports_0 = 'authored'</script>"#,
        r#"<script setup>import { ref as _imports_0 } from 'vue'</script>"#,
        r#"<script>function _imports_0() {} export default {}</script>"#,
        r#"<script>class _imports_0 {} export default {}</script>"#,
        r#"<script>import * as _imports_0 from 'vue'; export default {}</script>"#,
        r#"<script setup>const \u005fimports_0 = 'authored'</script>"#,
        r#"<script setup lang="ts">const _imports_0: string = 'authored'</script>"#,
        r#"<script setup lang="tsx">const _imports_0 = <div /></script>"#,
        r#"<script setup lang="jsx">const _imports_0 = <div /></script>"#,
    ];
    for script in scripts {
        for newline in ["\n", "\r\n"] {
            let source = [script, newline, "<template><img src='./a.svg'></template>"].concat();
            assert_eq!(names(&source), ["_imports_1"], "{source}");
        }
    }
}

#[test]
fn reserves_free_values_without_reserving_properties_strings_or_unrelated_locals() {
    for script in [
        "export default {read: () => _imports_0}",
        "export default {read: () => \\u005fimports_0}",
        "function write() { _imports_0 = 1 } export default {write}",
    ] {
        let source = [
            "<script>",
            script,
            "</script><template><img src='./a.svg'></template>",
        ]
        .concat();
        assert_eq!(names(&source), ["_imports_1"], "{source}");
    }
    for script in [
        "// _imports_0\nexport default {}",
        "const label = '_imports_0'; export default {label}",
        "const obj = { _imports_0: 1 }; export default {obj}",
        "export default {read: obj => obj._imports_0}",
        "function read(_imports_0) { return _imports_0 } export default {read}",
        "{ let _imports_0 = 1; console.log(_imports_0) } export default {}",
        "const _imports_00 = 1; export default {}",
    ] {
        let source = [
            "<script>",
            script,
            "</script><template><img src='./a.svg'></template>",
        ]
        .concat();
        assert_eq!(names(&source), ["_imports_0"], "{source}");
    }
    let source = "<script setup lang='ts'>type _imports_0 = string; import type { _imports_1 } from './types'</script><template><img src='./a.svg'></template>";
    assert_eq!(names(source), ["_imports_0"]);
}

#[test]
fn reserves_decoded_template_aliases_callbacks_and_global_values() {
    for template in [
        "<img v-for='_imports_0 in items' src='./a.svg' :title='_imports_0'>",
        "<img v-for='_imports_0 of items' src='./a.svg' :title='_imports_0'>",
        "<img v-for='({ value: _imports_0 }, index) of items' src='./a.svg'>",
        "<img v-for='({ a, b, c, d, e, f, g, h, i, j, k, value: _imports_0 }, index) of items' src='./a.svg'>",
        "<img v-for='(, _imports_0, index) of items' src='./a.svg'>",
        "<img v-for='&#95;imports_0 of items' src='./a.svg'>",
        r#"<img v-for='\u005fimports_0 of items' src='./a.svg'>"#,
        "<Child v-slot='{ _imports_0 }'><img src='./a.svg' :title='_imports_0'></Child>",
        "<Child v-slot='{ value: &#95;imports_0 }'><img src='./a.svg'></Child>",
        "<Child :render='_imports_0 => h(\"img\", {src: \"./a.svg\"})'><img src='./a.svg'></Child>",
        "<img src='./a.svg' @click='_imports_0++; go()'>",
        "<img src='./a.svg'>{{ _imports_0 }}",
    ] {
        let source = ["<template>", template, "</template>"].concat();
        assert_eq!(names(&source), ["_imports_1"], "{source}");
    }
    for template in [
        "<img src='./a.svg' title='_imports_0'>",
        "<img src='./a.svg' :title='obj._imports_0'>",
        "<img src='./a.svg' :title='\"_imports_0\"'>",
        "<img v-for='({ _imports_0: value }) of items' src='./a.svg'>",
        "<img src='./a.svg' :title='value as _imports_0'>",
    ] {
        let source = ["<template>", template, "</template>"].concat();
        assert_eq!(names(&source), ["_imports_0"], "{source}");
    }
}

#[test]
fn preserves_order_dedup_urls_and_chooses_each_next_free_index() {
    let source = "<script>const _imports_0 = 0, _imports_2 = 2; export default {}</script><script setup>const _imports_4 = 4</script><template><img src='./a.svg?raw#x'><img src='./b.svg'><img src='./a.svg?raw#x'><img src='./c.svg'><img src='./d.svg'></template>";
    let assets = collect_template_asset_urls(source, None, Some("App.vue"));
    assert_eq!(
        names(source),
        ["_imports_1", "_imports_3", "_imports_5", "_imports_6"]
    );
    assert_eq!(
        assets
            .iter()
            .map(|asset| asset.url.as_str())
            .collect::<Vec<_>>(),
        ["./a.svg?raw#x", "./b.svg", "./c.svg", "./d.svg"]
    );
    assert_eq!(
        assets,
        collect_template_asset_urls(source, None, Some("App.vue"))
    );
    assert!(
        names("<script>const _imports_0 = 0</script><template><p>no asset</p></template>")
            .is_empty()
    );
}
