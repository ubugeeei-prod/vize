//! Original plugin host packets, source anchors and Doctor component scores.
#![expect(
    clippy::unwrap_used,
    reason = "test fixture reads and results are asserted"
)]
use davinci_harness::alloc::CountingAllocator;
use std::alloc::System;
use vize::plugin_template::{PluginTemplateBatch, PluginTemplateProducts, analyze};
use vize_croquis_cf::TemplateComplexity;
use vize_l0::fact::FactManager;
use vize_patina::plugin_batch::{
    batch::{PluginSpec, build_batch, diagnostics},
    document::PluginDocument,
    facts::REGISTRY,
};
#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn original_plugin(
    source: &str,
    name: &str,
    spec: &PluginSpec<'_>,
) -> Result<PluginTemplateBatch, vize_patina::plugin_batch::error::HostError> {
    PluginDocument::build(source, name).and_then(|document| {
        let batch = build_batch(&document, spec, &mut FactManager::new(&REGISTRY))?;
        Ok(PluginTemplateBatch { batch, document })
    })
}

fn separate(source: &str, name: &str, spec: &PluginSpec<'_>) -> PluginTemplateProducts {
    PluginTemplateProducts {
        plugin: original_plugin(source, name, spec),
        complexity: TemplateComplexity::from_sfc(source),
    }
}

fn compare(actual: &PluginTemplateProducts, old: &PluginTemplateProducts, name: &str) {
    assert_eq!(actual.complexity, old.complexity, "complexity {name}");
    match (&actual.plugin, &old.plugin) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a.document.filename, b.document.filename, "{name}");
            assert_eq!(a.document.source, b.document.source, "{name}");
            assert_eq!(
                a.document.nodes, b.document.nodes,
                "SDK node, parent and span {name}"
            );
            assert_eq!(
                a.document.scopes, b.document.scopes,
                "scope bindings {name}"
            );
            assert_eq!(a.batch.nodes, b.batch.nodes, "{name}");
            assert_eq!(
                a.batch.json.as_bytes(),
                b.batch.json.as_bytes(),
                "batch {name}"
            );
            if let Some(last) = a.document.nodes.last() {
                let reports = format!(
                    r#"[{{"node":{},"rule":"original","message":"雪","fix":"🌱"}}]"#,
                    last.id
                );
                assert_eq!(
                    diagnostics(&a.document, "reader", &reports).unwrap(),
                    diagnostics(&b.document, "reader", &reports).unwrap(),
                    "{name}"
                );
            }
        }
        (Err(a), Err(b)) => {
            assert_eq!(
                std::mem::discriminant(a),
                std::mem::discriminant(b),
                "{name}"
            );
            assert_eq!(a.to_string(), b.to_string(), "{name}");
        }
        _ => panic!("plugin acceptance on {name}"),
    }
}

fn spec<'a>(demands: &'a [String], visit: Option<&'a [String]>) -> PluginSpec<'a> {
    PluginSpec {
        name: "reader",
        version: "1",
        fingerprint: "fixture",
        visit,
        demands,
    }
}

fn directed() -> [&'static str; 10] {
    [
        "<script setup lang='ts'>const 雪 = 1</script>\r\n<template><main><section v-if='a && b'><li v-for='(item,i) in rows' :key='i'>{{ item }}</li></section><i v-else>🌱</i></main></template>",
        "<template><Outer><template #default='{ item }'><Inner :thing='item' @click.once='go'><template #footer>{{ end }}</template></Inner></template></Outer></template>",
        "<template><i v-if='a'/><i v-else-if='b && (c || d)'/><span v-else/></template>",
        "<script setup>const missing=1</script>",
        "<template lang='pug'>p hello</template>",
        "<template src='./outside.html'></template>",
        "<template><p @click='(' :x='bad['>{{ ( }}</p></template>",
        "<template><p/></template><template><span/></template>",
        "<template>before {{ name }} after</template>",
        "<template></template>",
    ]
}

#[test]
fn configured_facts_scopes_errors_and_report_positions_match() {
    let demands = [
        "templateScopes".into(),
        "bindings".into(),
        "component-usages".into(),
    ];
    let kinds = ["ui.for".into(), "ui.component".into(), "ui.on".into()];
    let invalid = ["unknown-group".into()];
    let invalid_kind = ["ui.unknown".into()];
    for source in directed() {
        for request in [
            spec(&[], None),
            spec(&demands, None),
            spec(&demands, Some(&kinds)),
            spec(&invalid, None),
            spec(&[], Some(&invalid_kind)),
        ] {
            compare(
                &analyze(source, "App.vue", &request),
                &separate(source, "App.vue", &request),
                source,
            );
        }
    }
    let failed = analyze(directed()[0], "App.vue", &spec(&invalid, None));
    assert!(failed.plugin.is_err() && failed.complexity.is_some());
}

fn originals(mut visit: impl FnMut(usize, &str, &str)) -> usize {
    let manifest = std::fs::read(std::env::var("VIZE_PLUGIN_TEMPLATE_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (index, raw) in manifest
        .split(|b| *b == 0)
        .filter(|v| !v.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(raw).unwrap();
        let contents = std::fs::read(root.join(name)).unwrap();
        if let Ok(source) = std::str::from_utf8(&contents) {
            visit(index, name, source);
        } else {
            invalid += 1;
        }
    }
    invalid
}

#[test]
#[ignore = "compare every original full SDK packet and Doctor component record"]
fn original_packets_and_component_records_match() {
    let demands = ["templateScopes".into()];
    let request = spec(&demands, None);
    let (mut inputs, mut packets, mut bytes, mut rows, mut contributors) = (0, 0, 0, 0, 0);
    let invalid = originals(|_, name, source| {
        let actual = analyze(source, name, &request);
        compare(&actual, &separate(source, name, &request), name);
        inputs += 1;
        if let Ok(batch) = actual.plugin {
            packets += 1;
            bytes += batch.batch.json.len();
            rows += batch.document.nodes.len();
        }
        if let Some(facts) = actual.complexity {
            contributors += facts.contributors.len();
        }
    });
    assert!(inputs > 0 && rows > 0 && contributors > 0);
    eprintln!(
        "plugin_products inputs={inputs} packets={packets} payload_bytes={bytes} node_records={rows} contributors={contributors} invalid={invalid}"
    );
}

mod profile;

#[cfg(feature = "plugin-project-facts")]
mod project;
