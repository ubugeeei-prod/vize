//! Actual full CLI descriptor admission, separate from raw count controls.

use super::analyze_sfc_for_cross_file;
use std::path::Path;

fn analyze(source: &str) -> super::CrossFileSfcAnalysis {
    analyze_sfc_for_cross_file(source, Path::new("src/pages/fixture.vue"))
        .expect("valid original SFC descriptor")
}

#[test]
fn full_inline_original_templates_and_static_page_meta_produce_private_facts() {
    for source in [
        "<template><main><slot /></main></template>",
        r#"<script setup lang="ts">definePageMeta({ layout: "bare" })</script><template><section id="auth-page">login</section></template>"#,
    ] {
        let captured = analyze(source);
        assert_eq!(
            captured.analysis.nuxt_static_render_script,
            Some(true),
            "{source}"
        );
        let template = captured
            .analysis
            .nuxt_render_template
            .expect("full original template");
        assert!(template.admitted, "{source}");
        assert!(!template.opaque_structure, "{source}");
    }
    let captured = analyze("<template></template>");
    let facts = captured
        .analysis
        .nuxt_render_template
        .expect("explicit empty template");
    assert!(facts.admitted);
    assert_eq!(facts.slot_outlets, 0);
}

#[test]
fn transitions_runtime_scripts_original_errors_and_external_inputs_stay_unknown() {
    for source in [
        // The specialized plain-script path has no original statement visitor.
        // Even an empty ordinary block must keep that producer scope unknown.
        r#"<script></script><script setup>definePageMeta({ pageTransition: false, layoutTransition: false })</script><template><main /></template>"#,
        r#"<script setup>definePageMeta({ pageTransition: { mode: "in-out" } })</script><template><div /></template>"#,
        "<script setup>definePageMeta({ layoutTransition: true })</script><template><div /></template>",
        "<script setup>const router = useRouter()</script><template><div /></template>",
        "<script setup>definePageMeta({}); const =</script><template><div /></template>",
        "<script>const value = 1</script><script setup>definePageMeta({})</script><template><div /></template>",
        r#"<script setup src="./meta.ts"></script><template><div /></template>"#,
        r#"<template src="./content.html"></template>"#,
        r#"<template lang="pug">main</template>"#,
        "<template><div /></template><i18n>{}</i18n>",
        "<script setup>definePageMeta({})</script>",
    ] {
        let Some(captured) = analyze_sfc_for_cross_file(source, Path::new("src/pages/fixture.vue"))
        else {
            continue;
        };
        assert_ne!(
            captured.analysis.nuxt_static_render_script,
            Some(true),
            "{source}"
        );
    }
    if let Some(captured) = analyze_sfc_for_cross_file(
        "<template><div><</template>",
        Path::new("src/pages/broken.vue"),
    ) {
        assert!(
            !captured
                .analysis
                .nuxt_render_template
                .is_some_and(|facts| facts.admitted)
        );
    }
}

#[test]
fn slot_multiplicity_and_structural_wrappers_are_observed_without_reparsing() {
    for (source, slots, opaque) in [
        ("<template><slot /><slot /></template>", 2, false),
        (
            r#"<template><template v-for="n in 2"><slot /></template></template>"#,
            1,
            true,
        ),
        (r#"<template><slot v-if="visible" /></template>"#, 1, true),
        (r#"<template><slot :name="name" /></template>"#, 1, true),
        (
            r#"<template><NuxtLayout v-slot="scope"><NuxtPage /></NuxtLayout></template>"#,
            0,
            true,
        ),
    ] {
        let captured = analyze(source);
        let facts = captured
            .analysis
            .nuxt_render_template
            .expect("original full template");
        assert!(facts.admitted, "{source}");
        assert_eq!(facts.slot_outlets, slots, "{source}");
        assert_eq!(facts.opaque_structure, opaque, "{source}");
    }
}
