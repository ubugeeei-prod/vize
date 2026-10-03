//! Original Vue 2 filter pieces, not a modern carrier or rewritten JS parse.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "source/AST regression goldens intentionally fail immediately on a violated law"
)]

use vize_l0::{Allocator, SourceRoot, cstr};
use vize_l1::dialect::vue2::text::TextBoundaryKind;
use vize_l1::dialect::{LegacyVueVersion, vue2::surface};
use vize_l1::render::check_fidelity;

type Facts<'a> = (&'a str, Vec<(&'a str, Vec<&'a str>)>);

fn facts<'a>(arena: &'a Allocator, source: &'a str) -> Facts<'a> {
    let parsed = surface::parse_component(arena, source).expect("small source");
    check_fidelity(parsed.tree()).expect("original bytes");
    let chain = parsed.bindings()[0]
        .admitted()
        .expect("pinned supported family");
    (
        chain.base().source().text(),
        chain
            .filters()
            .iter()
            .map(|filter| {
                (
                    filter.name().text(),
                    filter
                        .arguments()
                        .iter()
                        .map(|arg| arg.source().text())
                        .collect(),
                )
            })
            .collect(),
    )
}

#[test]
fn top_level_facts_match_the_pinned_vue_2_7_16_grammar() {
    for (input, base, filters) in [
        ("{{ message | upper }}", "message", vec![("upper", vec![])]),
        (
            "{{ value | add(2) | wrap('雪', '後') }}",
            "value",
            vec![("add", vec!["2"]), ("wrap", vec!["'雪'", "'後'"])],
        ),
        (
            "{{ left || right | upper }}",
            "left || right",
            vec![("upper", vec![])],
        ),
        (
            "{{ (left | right) | number }}",
            "(left | right)",
            vec![("number", vec![])],
        ),
        ("{{ 'a|b' | upper }}", "'a|b'", vec![("upper", vec![])]),
        ("{{ \"a|b\" | upper }}", "\"a|b\"", vec![("upper", vec![])]),
        (
            "{{ `a|${value}` | upper }}",
            "`a|${value}`",
            vec![("upper", vec![])],
        ),
        (
            "{{ /a|b/.test(value) | number }}",
            "/a|b/.test(value)",
            vec![("number", vec![])],
        ),
        (
            "{{ value / 2 | number }}",
            "value / 2",
            vec![("number", vec![])],
        ),
        (
            "{{ list[index | mask] | number }}",
            "list[index | mask]",
            vec![("number", vec![])],
        ),
        (
            "{{ ({a: value | mask}) | json }}",
            "({a: value | mask})",
            vec![("json", vec![])],
        ),
        (
            "{{ value | pick(fn(1, 2), [3, 4], {a: 'x,y'}, /a,b/, `x,y`) }}",
            "value",
            vec![(
                "pick",
                vec!["fn(1, 2)", "[3, 4]", "{a: 'x,y'}", "/a,b/", "`x,y`"],
            )],
        ),
        ("{{ 雪 | 日本語 }}", "雪", vec![("日本語", vec![])]),
        (
            "{{ value | with-dash }}",
            "value",
            vec![("with-dash", vec![])],
        ),
        ("{{ value | upper() }}", "value", vec![("upper", vec![])]),
        ("{{ value | upper () }}", "value", vec![("upper ", vec![])]),
        ("{{ value }}", "value", vec![]),
        ("{{ value\r\n | upper }}", "value", vec![("upper", vec![])]),
        (
            "{{ &#160;雪&#160; | upper }}",
            "雪",
            vec![("upper", vec![])],
        ),
    ] {
        let arena = Allocator::default();
        assert_eq!(facts(&arena, input), (base, filters), "{input}");
    }
}

#[test]
fn authentic_nonzero_unicode_block_keeps_maps_and_actual_parser_owned_syntax() {
    let source = "前<script>const 雪=1</script><template>甲 {{ &#160;雪 | wrap('後', 1 + 2) }} 乙</template>後";
    let start = source.find("甲 ").unwrap();
    let end = source.find("</template>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let arena = Allocator::default();
    let parsed = surface::parse_component_block(&arena, block);
    assert_eq!(parsed.version(), LegacyVueVersion::V2);
    assert_eq!(parsed.block(), block);
    check_fidelity(parsed.tree()).unwrap();
    let binding = &parsed.bindings()[0];
    assert_eq!(
        &source[binding.span().start as usize..binding.span().end as usize],
        "{{ &#160;雪 | wrap('後', 1 + 2) }}"
    );
    let chain = binding.admitted().unwrap();
    let base = chain.base();
    assert_eq!(base.source().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(base.source().text(), "雪");
    let first = base.expression().unwrap();
    assert!(core::ptr::eq(first, base.expression().unwrap()));
    let ast_span = match first {
        oxc_ast::ast::Expression::Identifier(identifier) => identifier.span,
        _ => panic!("actual retained identifier AST"),
    };
    let span = base.authored_span(ast_span).unwrap();
    assert_eq!(&source[span.start as usize..span.end as usize], "雪");
    let filter = &chain.filters()[0];
    assert_eq!(
        &source[filter.span().start as usize..filter.span().end as usize],
        "wrap('後', 1 + 2)"
    );
    assert_eq!(filter.name().authored_root().as_ptr(), source.as_ptr());
    assert_eq!(filter.arguments().len(), 2);
    for argument in filter.arguments() {
        assert_eq!(argument.source().authored_root().as_ptr(), source.as_ptr());
        assert!(block.contains_block_span(argument.source().span()));
        assert!(argument.expression().is_some());
    }
}

#[test]
fn entity_decoding_selects_pipes_before_language_parse_with_exact_authored_cover() {
    let source = "{{ 雪 &#124; wrap(&quot;後&quot;) }}";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    let binding = &parsed.bindings()[0];
    assert!(binding.source().decode_map().is_some());
    let chain = binding.admitted().unwrap();
    assert_eq!(chain.base().source().text(), "雪");
    let argument = &chain.filters()[0].arguments()[0];
    assert_eq!(argument.source().text(), "\"後\"");
    assert_eq!(
        &source[argument.source().span().start as usize..argument.source().span().end as usize],
        "&quot;後&quot;"
    );
}

#[test]
fn language_holes_keep_original_diagnostics_and_later_bindings() {
    let source = "前 {{ 雪 + }} {{ value | wrap(1 +) }} {{ next | upper }} 後";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    assert_eq!(parsed.bindings().len(), 3);
    for syntax in [
        parsed.bindings()[0].chain().unwrap().base(),
        &parsed.bindings()[1].chain().unwrap().filters()[0].arguments()[0],
    ] {
        assert!(syntax.hole().is_some());
        assert!(syntax.expression().is_none());
        assert!(syntax.diagnostics().count() > 0);
        assert_eq!(syntax.source().authored_root().as_ptr(), source.as_ptr());
        for diagnostic in syntax.diagnostics() {
            for label in diagnostic.labels() {
                assert!(label.authored_span().is_ok());
            }
        }
    }
    assert!(parsed.bindings()[0].admitted().is_none());
    assert!(parsed.bindings()[1].admitted().is_none());
    assert!(parsed.bindings()[2].admitted().is_some());
    check_fidelity(parsed.tree()).unwrap();
}

#[test]
fn uncertified_families_are_typed_refusals_without_losing_source() {
    for (content, kind) in [
        ("", TextBoundaryKind::EmptyInterpolation),
        ("\u{00a0}", TextBoundaryKind::EmptyInterpolation),
        ("x\r y", TextBoundaryKind::HistoricalLineSeparator),
        ("x\u{2028}y", TextBoundaryKind::HistoricalLineSeparator),
        ("x\u{2029}y", TextBoundaryKind::HistoricalLineSeparator),
        (
            "/[a|b]/.test(x) | number",
            TextBoundaryKind::RegexCharacterClass,
        ),
        ("x /* pipe| */ | upper", TextBoundaryKind::CommentSyntax),
        (
            "x | wrap(...values)",
            TextBoundaryKind::UnsupportedArgumentList,
        ),
        ("x | wrap(1,,)", TextBoundaryKind::UnsupportedArgumentList),
        ("x | wrap(", TextBoundaryKind::UnbalancedFilterSyntax),
        ("x | \"bad\"", TextBoundaryKind::UnsupportedFilterName),
    ] {
        let source = cstr!("{{{{ {content} }}}} {{{{ next }}}}");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        assert_eq!(parsed.bindings().len(), 2, "{content}");
        assert_eq!(parsed.bindings()[0].boundaries()[0].kind, kind, "{content}");
        assert!(parsed.bindings()[0].admitted().is_none());
        assert!(parsed.bindings()[1].admitted().is_some());
        check_fidelity(parsed.tree()).unwrap();
    }
}

#[test]
fn only_literal_pre_suppresses_the_same_native_callback_visit() {
    for name in ["v-pre", "v-pre.foo", "v-pre:arg", "@pre", "v-previous"] {
        let source = cstr!("<div {name}>{{{{ value | upper }}}}</div>");
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, &source).unwrap();
        assert_eq!(
            parsed.bindings().len(),
            usize::from(name != "v-pre"),
            "{name}"
        );
        check_fidelity(parsed.tree()).unwrap();
    }
}

#[test]
fn all_unicode_prefixes_keep_authentic_source_and_valid_retained_spans() {
    let source = "前 <div>{{ 雪 | wrap('後', 1) }}</div> 乙 {{ next }}";
    for end in (0..=source.len()).filter(|end| source.is_char_boundary(*end)) {
        let input = &source[..end];
        let arena = Allocator::default();
        let parsed = surface::parse_component(&arena, input).unwrap();
        check_fidelity(parsed.tree()).unwrap();
        for binding in parsed.bindings() {
            assert!(parsed.block().contains_block_span(binding.span()));
            assert!(parsed.block().contains_block_span(binding.source().span()));
            if let Some(chain) = binding.chain() {
                assert!(
                    parsed
                        .block()
                        .contains_block_span(chain.base().source().span())
                );
                for filter in chain.filters() {
                    assert!(parsed.block().contains_block_span(filter.span()));
                    for argument in filter.arguments() {
                        assert!(parsed.block().contains_block_span(argument.source().span()));
                    }
                }
            }
        }
    }
}

#[test]
fn source_custody_and_both_projections_survive_scope_recovery() {
    let source = "前<div><button><button>{{ value | upper }}</button></button></div>後";
    let start = source.find("<div>").unwrap();
    let end = source.rfind("後").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&source[start..end], start as u32)
        .unwrap();
    let arena = Allocator::default();
    let parsed = surface::parse_component_with_authored_block(&arena, block);
    assert_eq!(parsed.bindings().len(), 1);
    check_fidelity(parsed.tree()).unwrap();
    check_fidelity(parsed.authored().expect("recovered authored projection")).unwrap();
    assert_eq!(parsed.block().root_source().as_ptr(), source.as_ptr());
}

#[test]
fn encoded_delimiter_framing_is_an_explicit_authored_boundary() {
    let source = "前 &#123;&#123; value &#125;&#125; {{ next }}";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    assert_eq!(parsed.bindings().len(), 1);
    assert_eq!(parsed.unsupported().len(), 4);
    for (boundary, (start, end, entity)) in parsed.unsupported().iter().zip([
        (4, 10, "&#123;"),
        (10, 16, "&#123;"),
        (23, 29, "&#125;"),
        (29, 35, "&#125;"),
    ]) {
        assert_eq!(boundary.kind, TextBoundaryKind::EncodedDelimiter);
        assert!(parsed.block().contains_block_span(boundary.span));
        assert_eq!((boundary.span.start, boundary.span.end), (start, end));
        assert_eq!(&source[start as usize..end as usize], entity);
    }
    assert!(parsed.bindings()[0].admitted().is_some());
    check_fidelity(parsed.tree()).unwrap();
}

#[test]
fn a_later_filter_refusal_preserves_every_started_language_observation() {
    let source = "{{ 雪 + | wrap(1 +) | \"bad\" }} {{ next }}";
    let arena = Allocator::default();
    let parsed = surface::parse_component(&arena, source).unwrap();
    let binding = &parsed.bindings()[0];
    assert_eq!(
        binding.boundaries()[0].kind,
        TextBoundaryKind::UnsupportedFilterName
    );
    assert!(binding.admitted().is_none());
    let partial = binding.chain().unwrap();
    assert!(partial.base().diagnostics().count() > 0);
    assert_eq!(partial.filters().len(), 1);
    assert!(partial.filters()[0].arguments()[0].diagnostics().count() > 0);
    assert!(parsed.bindings()[1].admitted().is_some());
    check_fidelity(parsed.tree()).unwrap();
}
