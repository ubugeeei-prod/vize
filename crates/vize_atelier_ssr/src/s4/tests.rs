use super::{LegacyReason, SsrS4Request, SsrS4Selection, select_ssr_lane};
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::Allocator;

fn select(source: &str, options: &SsrCompilerOptions) -> SsrS4Selection {
    let allocator = Allocator::new();
    let experimental = SsrCompilerExperimentalOptions::default();
    select_ssr_lane(
        &allocator,
        source,
        &SsrS4Request {
            options,
            experimental: &experimental,
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
        },
    )
}

#[test]
fn admitted_templates_are_emitted_from_the_string_plan() {
    let selection = select(
        r#"<section><h1>{{ title }}</h1><p>Ready</p></section>"#,
        &SsrCompilerOptions::default(),
    );
    let SsrS4Selection::Emitted(result) = selection else {
        panic!("expected plan emission, got {selection:?}");
    };
    assert_eq!(
        result.code.as_str(),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<section${_ssrRenderAttrs(_attrs)}><h1>${_ssrInterpolate(_ctx.title)}</h1><p>Ready</p></section>`)\n}\n"
    );
    assert_eq!(
        result.preamble.as_str(),
        "import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs } from \"@vue/server-renderer\"\n"
    );
}

#[test]
fn unsupported_bridge_options_select_legacy_before_lowering() {
    let selection = select(
        r#"<div><!-- kept --></div>"#,
        &SsrCompilerOptions {
            comments: true,
            ..SsrCompilerOptions::default()
        },
    );
    assert!(matches!(
        selection,
        SsrS4Selection::Legacy(LegacyReason::Options)
    ));
}

#[test]
fn a_croquis_summary_names_its_own_legacy_reason() {
    let selection = select(
        r#"<div>{{ msg }}</div>"#,
        &SsrCompilerOptions {
            croquis: Some(Box::default()),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrS4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );
}

#[test]
fn unsupported_emission_options_select_legacy_after_the_witness() {
    let selection = select(
        r#"<div>{{ msg }}</div>"#,
        &SsrCompilerOptions {
            inline: true,
            ..SsrCompilerOptions::default()
        },
    );
    assert!(matches!(
        selection,
        SsrS4Selection::Legacy(LegacyReason::Options)
    ));
}

#[test]
fn unowned_shapes_name_their_legacy_reason() {
    let cases = [
        (
            r#"<Foo><div v-if="a"><template #a>x</template></div></Foo>"#,
            LegacyReason::Operation,
        ),
        (
            r#"<Foo><template #[names[0]]>x</template></Foo>"#,
            LegacyReason::Operation,
        ),
        (r#"<Foo v-model:[a]="x" />"#, LegacyReason::Binding),
        (r#"<div v-focus="ok"></div>"#, LegacyReason::Binding),
        (r#"<div v-once>{{ a }}</div>"#, LegacyReason::Binding),
        (r#"<script>x</script>"#, LegacyReason::Element),
        (
            "<div><!-- @vize:forget pre-escaped --><p>y</p></div>",
            LegacyReason::SurfaceSemantics,
        ),
        (
            "<p v-if=\"a\">1</p><!-- @vize:todo x --><p v-else>2</p>",
            LegacyReason::SurfaceSemantics,
        ),
        (r#"<input v-model:foo="x">"#, LegacyReason::Binding),
        (r#"<div :id.camel="x"></div>"#, LegacyReason::Binding),
        (r#"<div :[key]="x"></div>"#, LegacyReason::Binding),
        (
            r#"<div>{{ a &amp;&amp; b }}</div>"#,
            LegacyReason::ExpressionOrEncoding,
        ),
    ];
    for (source, expected) in cases {
        let selection = select(source, &SsrCompilerOptions::default());
        assert!(
            matches!(selection, SsrS4Selection::Legacy(reason) if reason == expected),
            "{source}: expected {expected:?}, got {selection:?}"
        );
    }
}
