use vize_l0::{String, cstr};

pub struct Case {
    pub name: &'static str,
    pub files: Vec<(&'static str, String)>,
}

fn sfc(script: &str, template: &str) -> String {
    cstr!("<script setup lang=\"ts\">\n{script}\n</script>\n<template>{template}</template>\n")
}

pub fn cases() -> Vec<Case> {
    let shared = sfc("defineProps<{ label?: string }>()", "<p>日本語🙂</p>");
    vec![
        Case {
            name: "shared-child-late-resolution",
            files: vec![
                ("First.vue", sfc("import Alias from './Shared.vue'", "<Alias label=\"first\" />")),
                ("Shared.vue", shared.clone()),
                ("Last.vue", sfc("import Other from './Shared.vue'", "<Other label=\"last\" />")),
            ],
        },
        Case {
            name: "reexport-and-import-alias",
            files: vec![
                ("First.vue", sfc("import Alias from './barrel'", "<Alias />")),
                ("barrel.ts", "export { default } from './Shared.vue'".into()),
                ("Shared.vue", shared),
                ("Last.vue", sfc("import Other from './Shared.vue'", "<Other />")),
            ],
        },
        Case {
            name: "missing-and-type-only-imports",
            files: vec![
                ("App.vue", sfc("import Missing from './Absent.vue'\nimport type { Item } from './types'", "<Missing />")),
                ("types.ts", "export interface Item { value: string }".into()),
            ],
        },
        Case {
            name: "component-and-module-cycle",
            files: vec![
                ("First.vue", sfc("import Last from './Last.vue'", "<Last />")),
                ("Last.vue", sfc("import First from './First.vue'", "<First />")),
            ],
        },
        Case {
            name: "owner-edited-import-and-component-removed",
            files: vec![
                ("First.vue", edited_source()),
                ("Last.vue", sfc("import First from './First.vue'", "<First />")),
            ],
        },
        Case {
            name: "dependency-deleted-unresolved-owner",
            files: vec![
                ("First.vue", sfc("import Last from './Last.vue'", "<Last />")),
            ],
        },
        Case {
            name: "dependency-renamed-import-repaired",
            files: vec![
                ("First.vue", sfc("import Renamed from './Renamed.vue'", "<Renamed />")),
                ("Renamed.vue", sfc("import First from './First.vue'", "<First />")),
            ],
        },
        Case {
            name: "nuxt-implicit-layout-and-page",
            files: vec![
                ("app.vue", "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>".into()),
                ("layouts/default.vue", "<template><section><slot /></section></template>".into()),
                ("pages/index.vue", "<template><p>日本語🙂</p></template>".into()),
            ],
        },
        Case {
            name: "mixed-jsx-ts-and-crlf",
            files: vec![
                ("First.vue", "<script lang=\"jsx\">\nimport Alias from './Last.vue'\nconst view = <i />\nexport default {}\n</script>\n<script setup lang=\"ts\">const value: number = 1</script>\n<template><Alias /></template>".replace('\n', "\r\n").into()),
                ("Last.vue", "<script setup lang=\"tsx\">const view = <b>日本語🙂</b></script><template><p /></template>".into()),
            ],
        },
    ]
}

pub fn orders(count: usize) -> Vec<Vec<usize>> {
    let original: Vec<_> = (0..count).collect();
    let mut result = Vec::new();
    for reverse in [false, true] {
        for shift in 0..count {
            let mut order = original.clone();
            if reverse {
                order.reverse();
            }
            order.rotate_left(shift);
            if !result.contains(&order) {
                result.push(order);
            }
        }
    }
    result
}

pub fn edited_source() -> String {
    sfc(
        "const label = 'dependency removed 日本語🙂'",
        "<p>{{ label }}</p>",
    )
}
