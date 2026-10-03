//! Original Vue 2 argument windows, including complete blank and trailing lists.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "original-source argument laws fail immediately on violated golden facts"
)]

use vize_l0::{Allocator, SourceRoot, cstr};
use vize_l1::dialect::vue2::{surface, text::TextBoundaryKind};
use vize_l1::render::check_fidelity;

#[test]
fn blank_and_single_trailing_windows_match_original_ordered_argument_facts() {
    for (content, name, expected) in [
        ("x | upper( )", "upper", vec![]),
        ("x | upper(\t\n\u{00a0})", "upper", vec![]),
        ("x | upper(&#160;&#xfeff;)", "upper", vec![]),
        ("x | upper ( )", "upper ", vec![]),
        ("x | wrap(1,)", "wrap", vec!["1"]),
        ("x | add(2,  )", "add", vec!["2"]),
        ("x | add(2&#44;)", "add", vec!["2"]),
        (
            "x | pick((1,2), [3,], {a:3,},)",
            "pick",
            vec!["(1,2)", "[3,]", "{a:3,}"],
        ),
    ] {
        let source = cstr!("{{{{ {content} }}}} {{{{ next }}}}");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        assert_eq!(parsed.bindings().len(), 2, "{content}");
        let chain = parsed.bindings()[0].admitted().expect("pinned list family");
        assert_eq!(chain.base().source().text(), "x");
        assert_eq!(chain.filters().len(), 1);
        let filter = &chain.filters()[0];
        assert_eq!(filter.name().text(), name, "{content}");
        assert_eq!(
            filter
                .arguments()
                .iter()
                .map(|argument| argument.source().text())
                .collect::<Vec<_>>(),
            expected,
            "{content}"
        );
        assert!(
            filter
                .arguments()
                .iter()
                .all(|argument| argument.expression().is_some())
        );
        assert!(parsed.bindings()[1].admitted().is_some());
        check_fidelity(parsed.tree()).unwrap();
    }
}

#[test]
fn encoded_trailing_punctuation_preserves_nonzero_original_argument_custody() {
    let source = "前<script>const 雪=1</script><template>甲 {{ 雪 | pick(&quot;後&quot;&#44; &#160;) | upper( ) }} 乙</template>後";
    let start = source.find("甲 ").unwrap();
    let end = source.find("</template>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let arena = Allocator::default();
    let parsed = surface::parse_component_block(&arena, block);
    let chain = parsed.bindings()[0].admitted().unwrap();
    assert_eq!(chain.filters().len(), 2);
    let filter = &chain.filters()[0];
    assert_eq!(filter.name().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(
        &source[filter.span().start as usize..filter.span().end as usize],
        "pick(&quot;後&quot;&#44; &#160;)"
    );
    assert_eq!(filter.arguments().len(), 1);
    let argument = &filter.arguments()[0];
    assert_eq!(argument.source().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(argument.source().text(), "\"後\"");
    assert!(argument.source().decode_map().is_some());
    assert_eq!(
        &source[argument.source().span().start as usize..argument.source().span().end as usize],
        "&quot;後&quot;"
    );
    let ast = argument.expression().unwrap();
    assert!(core::ptr::eq(ast, argument.expression().unwrap()));
    let ast_span = match ast {
        oxc_ast::ast::Expression::StringLiteral(literal) => literal.span,
        _ => panic!("actual original string literal AST"),
    };
    assert_eq!(
        argument.authored_span(ast_span).unwrap(),
        argument.source().span()
    );
    assert!(block.contains_block_span(argument.source().span()));
    assert!(chain.filters()[1].arguments().is_empty());
    check_fidelity(parsed.tree()).unwrap();
}

#[test]
fn list_holes_remain_structural_refusals_and_nonwhitespace_remains_native_syntax() {
    for content in [
        "x | upper(,)",
        "x | upper(, )",
        "x | add(2,,)",
        "x | add(2, ,)",
        "x | add(2,,3,)",
    ] {
        let source = cstr!("{{{{ {content} }}}} {{{{ next }}}}");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        let binding = &parsed.bindings()[0];
        assert_eq!(
            binding.boundaries()[0].kind,
            TextBoundaryKind::UnsupportedArgumentList
        );
        assert!(binding.admitted().is_none());
        assert!(binding.chain().unwrap().filters().is_empty());
        assert!(parsed.bindings()[1].admitted().is_some());
        check_fidelity(parsed.tree()).unwrap();
    }
    for content in ["x | add(1 +,)", "x | upper(\u{0085})"] {
        let source = cstr!("{{{{ {content} }}}} {{{{ next }}}}");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        let binding = &parsed.bindings()[0];
        assert!(binding.boundaries().is_empty());
        let argument = &binding.chain().unwrap().filters()[0].arguments()[0];
        assert!(argument.hole().is_some());
        assert!(argument.diagnostics().count() > 0);
        assert_eq!(argument.source().authored_root().as_ptr(), source.as_ptr());
        assert!(binding.admitted().is_none());
        assert!(parsed.bindings()[1].admitted().is_some());
    }
}

#[test]
fn a_later_list_refusal_keeps_every_started_original_ast_and_diagnostic() {
    let source = "{{ 雪 + | wrap(1 +,) | add(2,,) }} {{ next }}";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    let binding = &parsed.bindings()[0];
    assert_eq!(
        binding.boundaries()[0].kind,
        TextBoundaryKind::UnsupportedArgumentList
    );
    assert!(binding.admitted().is_none());
    let chain = binding.chain().unwrap();
    assert!(chain.base().diagnostics().count() > 0);
    assert_eq!(chain.filters().len(), 1);
    assert!(chain.filters()[0].arguments()[0].diagnostics().count() > 0);
    assert!(parsed.bindings()[1].admitted().is_some());
    check_fidelity(parsed.tree()).unwrap();
}
