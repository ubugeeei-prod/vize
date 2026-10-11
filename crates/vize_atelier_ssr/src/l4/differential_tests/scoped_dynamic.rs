//! Own the real dynamic template without falling back to the legacy walker.

use super::{SsrCompilerExperimentalOptions, SsrCompilerOptions, SsrL4Selection};
use crate::compile::{SsrLane, compile_ssr_on_lane};
use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
use vize_l0::Allocator;

#[test]
fn selected_scoped_dynamic_templates_preserve_complete_module_and_map_custody() {
    let sfc = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/VFHeading.vue.txt"
    ));
    let descriptor =
        vize_atelier_sfc::parse_sfc(sfc, Default::default()).expect("whole unchanged original SFC");
    let heading = descriptor
        .template
        .as_ref()
        .expect("original template")
        .content
        .as_ref();
    let options = SsrCompilerOptions {
        scope_id: Some("data-v-ea49ded1".into()),
        ..Default::default()
    };
    let experimental = SsrCompilerExperimentalOptions {
        source_map: true,
        source_map_filename: Some("VFHeading.vue".into()),
        ..Default::default()
    };
    for (name, source) in [
        ("whole-original-heading", heading),
        ("dynamic-root", r#"<component :is="tag" />"#),
        (
            "dynamic-nested",
            r#"<div><component :is="tag" id="kept"><span>Logo</span></component></div>"#,
        ),
        (
            "dynamic-spread",
            r#"<component :is="tag" v-bind="$attrs"><slot /></component>"#,
        ),
    ] {
        assert!(
            matches!(
                super::selection(source, &options, &experimental),
                SsrL4Selection::Emitted(_)
            ),
            "native L4 must own {name}"
        );
        super::assert_parity(source, &options, &experimental, name);
    }
}

#[test]
fn static_component_scope_and_fallthrough_keep_the_complete_prior_modules() {
    let options = SsrCompilerOptions {
        scope_id: Some("data-v-control".into()),
        ..Default::default()
    };
    for (name, source) in [
        (
            "static-root-scope",
            r#"<NuxtLink to="/news" class="news__link"><span>News</span></NuxtLink>"#,
        ),
        (
            "static-nested-scope",
            r#"<div><Foo id="child" v-bind="$attrs" /></div>"#,
        ),
    ] {
        assert!(matches!(
            super::selection(source, &options, &Default::default()),
            SsrL4Selection::Emitted(_)
        ));
        super::assert_parity(source, &options, &Default::default(), name);
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            Default::default(),
            true,
            SsrLane::Selected,
        );
        assert!(errors.is_empty());
        insta::assert_snapshot!(
            name,
            std::format!(
                "preamble:\n{}\ncode:\n{}\nmap:{:?}",
                result.preamble,
                result.code,
                result.map
            )
        );
    }
}
