//! #7892: root CSS binds agree on selected and retained template emitters.
use vize_atelier_ssr::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions,
    compile_ssr_with_template_syntax_and_experimental_options,
};
use vize_l0::Allocator;

const TEMPLATES: &[&str] = &[
    "<p>one</p><p>two</p>",
    "<div><p>one</p></div><p>two</p>",
    "<p v-if=\"color === 'red'\">one</p><p>two</p>",
    "<template v-if=\"color === 'red'\"><p>one</p><p>two</p></template>",
    "<RootLeaf>one</RootLeaf><RootLeaf>two</RootLeaf>",
    "<p v-for=\"n in [1,2]\">{{ n }}</p>",
];

#[test]
fn fragment_css_binds_preserve_maps_and_nested_root_boundaries() {
    for template in TEMPLATES {
        let allocator = Allocator::new();
        let options = SsrCompilerOptions {
            ssr_css_vars: Some("{ \":--fixture-color\": (_ctx.color) }".into()),
            ..Default::default()
        };
        let (_, errors, plain) = compile_ssr_with_template_syntax_and_experimental_options(
            &allocator,
            template,
            options.clone(),
            Default::default(),
            Default::default(),
        );
        assert!(errors.is_empty(), "{errors:?}");
        let (_, errors, mapped) = compile_ssr_with_template_syntax_and_experimental_options(
            &allocator,
            template,
            options,
            Default::default(),
            SsrCompilerExperimentalOptions {
                source_map: true,
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(mapped.code, plain.code);
        assert_eq!(mapped.preamble, plain.preamble);
        assert!(mapped.map.is_some());
        let calls = plain.code.matches("_ssrRenderAttrs(_cssVars)").count();
        if *template == "<p>one</p><p>two</p>" {
            assert_eq!(calls, 2);
        } else if template.contains("v-for") {
            assert_eq!(calls, 0, "Vue does not inject root-loop CSS binds");
        } else if template.starts_with("<div>") {
            assert_eq!(calls, 2, "only the div and sibling p, not nested p");
        }
    }
}

#[cfg(feature = "legacy-differential")]
#[test]
fn selected_and_retained_emitters_have_identical_complete_css_results() {
    for template in TEMPLATES {
        let pair = vize_atelier_ssr::differential::compare_ssr_lanes(
            template,
            &SsrCompilerOptions {
                ssr_css_vars: Some("{ \":--fixture-color\": (_ctx.color) }".into()),
                ..Default::default()
            },
            &SsrCompilerExperimentalOptions {
                source_map: true,
                ..Default::default()
            },
        );
        assert_eq!(pair.lane, "s4", "actual native admission: {template}");
        assert!(pair.selected.errors.is_empty());
        assert!(pair.legacy.errors.is_empty());
        assert_eq!(
            pair.selected.result.code, pair.legacy.result.code,
            "{template}"
        );
        assert_eq!(
            pair.selected.result.preamble, pair.legacy.result.preamble,
            "{template}"
        );
        assert_eq!(
            pair.selected.result.map, pair.legacy.result.map,
            "{template}"
        );
    }
}
