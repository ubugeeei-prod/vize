use super::super::SsrCodegenContext;
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_l0::{Allocator, Span};

#[test]
fn wrapped_fragments_keep_complete_text_links_and_maps() {
    let allocator = Allocator::default();
    let options = SsrCompilerOptions::default();
    let source = "<p>{{ a + 雪 }}</p>";
    for source_map in [false, true] {
        let ctx = SsrCodegenContext::new_with_experimental_options(
            &allocator,
            &options,
            source,
            SsrCompilerExperimentalOptions {
                source_map,
                ..SsrCompilerExperimentalOptions::default()
            },
        );
        for (code, span) in [
            ("_ctx.a + _ctx.雪", Span::new(6, 13)),
            ("1 + _ctx.雪", Span::new(6, 13)),
            ("", Span::new(6, 13)),
            ("_ctx.a", Span::new(0, 0)),
        ] {
            for (prefix, suffix) in [
                ("", ""),
                ("_ssrInterpolate(", ")"),
                ("xxxxxxxxxxxxxxxxxxxxxxxx", ")"),
                ("xxxxxxxxxxxxxxxxxxxxxxxxx", "雪"),
            ] {
                // Retained two-document composition is the complete reference,
                // including link names/segment flags and serialized source maps.
                let mut expression = EmitDocument::default();
                if source_map {
                    expression.push_expression(code, span, source);
                } else {
                    expression.push_str(code);
                }
                let mut expected = EmitDocument::plain(prefix);
                expected.push_spanned(&expression);
                expected.push_str(suffix);
                let actual = ctx.spanned_wrap(prefix, code, suffix, span);
                assert_eq!(actual, expected);
                assert_eq!(actual.as_str(), [prefix, code, suffix].concat());
                assert_eq!(
                    actual.source_map("wrapped.vue", source),
                    expected.source_map("wrapped.vue", source),
                );
            }
        }
    }
}
