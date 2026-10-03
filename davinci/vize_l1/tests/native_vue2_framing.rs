//! Vue 2 selects default delimiter framing after decoding complete HTML text.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "whole original-source framing goldens fail immediately on a violated law"
)]

use vize_l0::{Allocator, SourceRoot, cstr};
use vize_l1::dialect::vue2::{surface, text::TextBoundaryKind};
use vize_l1::render::check_fidelity;

#[test]
fn encoded_historical_separators_are_refused_before_expression_edge_trim() {
    for entity in ["&#x2028;", "&#8233;", "&#13;"] {
        for content in [
            cstr!("{entity}value"),
            cstr!("value{entity}+ 1"),
            cstr!("value{entity}"),
        ] {
            let source = cstr!("前<script>const 雪=1</script>{{{{ {content} }}}} {{{{ next }}}}後");
            let start = source.find("{{").unwrap();
            let end = source.rfind("後").unwrap();
            let block = SourceRoot::new(&source)
                .unwrap()
                .block(&source[start..end], start as u32)
                .unwrap();
            let arena = Allocator::default();
            let parsed = surface::parse_component_block(&arena, block);
            assert_eq!(parsed.bindings().len(), 2, "{content}");
            let refused = &parsed.bindings()[0];
            assert_eq!(refused.boundaries().len(), 1, "{content}");
            assert_eq!(
                refused.boundaries()[0].kind,
                TextBoundaryKind::HistoricalLineSeparator,
                "{content}"
            );
            assert!(refused.chain().is_none(), "{content}");
            assert_eq!(refused.source().authored_root().as_ptr(), source.as_ptr());
            assert_eq!(
                &source[refused.span().start as usize..refused.span().end as usize],
                cstr!("{{{{ {content} }}}}").as_str()
            );
            assert!(block.contains_block_span(refused.source().span()));
            assert!(parsed.bindings()[1].admitted().is_some());
            check_fidelity(parsed.tree()).unwrap();
        }
    }
}

#[test]
fn complete_decoded_window_preserves_encoded_and_mixed_crlf_framing() {
    for (content, expression) in [
        ("&#13;&#10;value", "value"),
        ("value&#13;&#10;", "value"),
        ("value&#13;&#10;+ 1", "value\r\n+ 1"),
        ("&#13;\nvalue", "value"),
        ("value&#13;\n", "value"),
        ("\r&#10;value", "value"),
        ("value\r&#10;", "value"),
    ] {
        let source = cstr!("前 {{{{ {content} }}}} {{{{ next }}}} 後");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        assert_eq!(parsed.bindings().len(), 2, "{content}");
        let binding = &parsed.bindings()[0];
        assert!(binding.boundaries().is_empty(), "{content}");
        let syntax = binding.admitted().expect("complete CRLF framing").base();
        assert_eq!(syntax.source().text(), expression, "{content}");
        assert_eq!(syntax.source().authored_root().as_ptr(), source.as_ptr());
        let ast = syntax.expression().unwrap();
        assert!(core::ptr::eq(ast, syntax.expression().unwrap()));
        assert!(parsed.bindings()[1].admitted().is_some());
        check_fidelity(parsed.tree()).unwrap();
    }
}

#[test]
fn full_text_preparation_never_decodes_a_generated_reference_twice() {
    let source = "前 {{ '&amp;#x2028;' }} 後";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    let binding = &parsed.bindings()[0];
    let syntax = binding.admitted().expect("literal reference string").base();
    assert_eq!(syntax.source().text(), "'&#x2028;'");
    assert!(syntax.source().decode_map().is_some());
    assert_eq!(syntax.source().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(
        &source[syntax.source().span().start as usize..syntax.source().span().end as usize],
        "'&amp;#x2028;'"
    );
    check_fidelity(parsed.tree()).unwrap();
}
