//! CSS references, client getters, and SSR inline values share one property name.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::{String, cstr};

#[test]
fn css_bind_expressions_resolve_in_client_getters_and_ssr_html() {
    let cases = [
        ("template-literal", "`${fontSize}px`", "18px"),
        ("function-call", "Math.max(fontSize, 12) + 'px'", "18px"),
        ("conditional", "fontSize > 12 ? 'large' : 'small'", "large"),
    ];
    let inputs = cases
        .into_iter()
        .flat_map(|case| [false, true].map(move |is_prod| (case, is_prod)))
        .map(|((name, expression, expected), is_prod)| {
            let source = cstr!(
                r#"<script setup>
import {{ ref }} from 'vue'
const fontSize = ref(18)
const fogOpacity = ref(0.5)
</script>
<template><div class="root">text</div></template>
<style scoped>
.root {{ --tg-font-size: v-bind("{expression}"); opacity: v-bind(fogOpacity); }}
</style>"#
            );
            let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("parse repro");
            let mut options = SfcCompileOptions {
                scope_id: Some("f82a533a".into()),
                ..Default::default()
            };
            options.script.id = Some("app/pages/index.vue".into());
            options.template.is_prod = is_prod;
            let client = compile_sfc(&descriptor, options.clone()).expect("compile client");
            options.template.ssr = true;
            let ssr = compile_sfc(&descriptor, options).expect("compile SSR");
            assert!(client.errors.is_empty(), "{:?}", client.errors);
            assert!(ssr.errors.is_empty(), "{:?}", ssr.errors);
            let pipeline = vize_atelier_sfc::vite_plugin::transform_css_vars_for_pipeline(
                &descriptor.styles[0].content,
                "data-v-f82a533a",
            );
            serde_json::json!({
                "name": name,
                "source": source,
                "client": client.code.as_str(),
                "ssr": ssr.code.as_str(),
                "css": client.css.expect("compiled CSS").as_str(),
                "pipeline": pipeline.as_str(),
                "expected": expected,
                "isProd": is_prod,
            })
        })
        .collect::<Vec<_>>();
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/css-bind-runtime.mjs"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run Vue runtime assertions");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(
            serde_json::to_string(&inputs)
                .expect("serialize cases")
                .as_bytes(),
        )
        .expect("write cases");
    let output = child.wait_with_output().expect("runtime results");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}
