//! Original Vue 1 callback/CST/expression ownership, without runtime credit.

use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::dialect::vue1::surface::{self, TextRefusal};
use vize_l1::dialect::vue1::text::{TextBinding, TextBoundaryKind};
use vize_l1::embed::{Grammar, Lang, Shape, syntax::EmbedHole};
use vize_l1::{SurfaceChild, check_fidelity};

#[test]
fn original_callback_cst_and_actual_expression_keep_nonzero_unicode_root_coordinates() {
    let arena = Allocator::default();
    let root = "前🍣<template><p>雪{{ /*keep*/ msg &amp;&amp; 条件 }}</p></template>後";
    let start = root.find("<p>").unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let owner = surface::parse_component_with_authored_block(&arena, block);
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    assert!(owner.errors().is_empty());
    assert_eq!(owner.bindings().len(), 1);
    let binding = &owner.bindings()[0];
    assert_eq!(
        binding.span().slice(root),
        "{{ /*keep*/ msg &amp;&amp; 条件 }}"
    );
    assert_eq!(binding.content_span().slice(root), binding.raw_content());
    assert_eq!(binding.raw_content(), " /*keep*/ msg &amp;&amp; 条件 ");
    let syntax = binding.syntax().unwrap();
    assert_eq!(
        syntax.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert!(syntax.source_type().is_module());
    assert!(!syntax.source_type().is_typescript());
    assert_eq!(syntax.source().authored_root().as_ptr(), root.as_ptr());
    assert_eq!(syntax.source().text(), "/*keep*/ msg && 条件");
    assert!(syntax.source().decode_map().is_some());
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(
        syntax
            .comments()
            .next()
            .unwrap()
            .authored_span()
            .unwrap()
            .slice(root),
        "/*keep*/"
    );
    assert_eq!(syntax.diagnostics().count(), 0);
    let original = syntax.expression().unwrap();
    assert_eq!(
        syntax.authored_span(original.span()).unwrap().slice(root),
        "msg &amp;&amp; 条件"
    );
    let parent = owner.children().next().unwrap();
    let mut children = parent.children().unwrap();
    assert_eq!(children.next().unwrap().ordinal(), 0);
    let child = children.next().unwrap();
    let actual = child.surface();
    let view = owner.text_for(child).unwrap();
    assert!(core::ptr::eq(view.child().surface(), actual));
    assert!(core::ptr::eq(view.child().component(), &owner));
    assert_eq!(view.child().ordinal(), 1);
    assert!(core::ptr::eq(view.expression(), original));
    assert!(core::ptr::eq(view.binding(), binding));
    assert_eq!(children.len(), 0);
    // The existing optional authored CST is created only after interactive
    // repair; requesting it does not manufacture a second wellformed tree.
    assert!(owner.authored_children().is_none());
}

#[test]
fn repeated_bytes_distinct_occurrences_and_same_buffer_foreign_parses_cannot_substitute() {
    let arena = Allocator::default();
    let source = String::from("{{ same }}{{ same }}");
    let owner = surface::parse_component(&arena, &source).unwrap();
    let foreign = surface::parse_component(&arena, &source).unwrap();
    assert_eq!(owner.bindings().len(), 2);
    assert_ne!(
        owner.bindings()[0].content_span(),
        owner.bindings()[1].content_span()
    );
    assert_ne!(
        owner.bindings()[0].raw_content().as_ptr(),
        owner.bindings()[1].raw_content().as_ptr()
    );
    let first = owner.text_for(owner.children().next().unwrap()).unwrap();
    let second = owner.text_for(owner.children().nth(1).unwrap()).unwrap();
    assert!(!core::ptr::eq(first.expression(), second.expression()));
    assert!(core::ptr::eq(first.binding(), &owner.bindings()[0]));
    assert!(core::ptr::eq(second.binding(), &owner.bindings()[1]));
    assert_eq!(
        owner
            .text_for(foreign.children().next().unwrap())
            .unwrap_err(),
        TextRefusal::ForeignComponent
    );
    let other_bytes = String::from(source.as_str());
    let other = surface::parse_component(&arena, &other_bytes).unwrap();
    assert_eq!(
        owner
            .text_for(other.children().next().unwrap())
            .unwrap_err(),
        TextRefusal::ForeignComponent
    );
}

#[test]
fn owner_moves_preserve_original_arena_ast_and_join_only_fresh_owner_views() {
    let arena = Allocator::default();
    let source = "{{ value }}";
    let owner = surface::parse_component(&arena, source).unwrap();
    let original = core::ptr::from_ref(owner.bindings()[0].syntax().unwrap().expression().unwrap());
    let mut parked = vec![owner];
    parked.reserve(64);
    let moved = core::hint::black_box(parked.pop().unwrap());
    let view = moved.text_for(moved.children().next().unwrap()).unwrap();
    assert_eq!(core::ptr::from_ref(view.expression()), original);
    assert_eq!(view.child().ordinal(), 0);
    assert!(view.child().parent_element().is_none());
    assert!(core::mem::needs_drop::<TextBinding<'_>>());
    assert!(core::mem::needs_drop::<surface::ComponentParse<'_>>());
}

#[test]
fn malformed_expression_retains_original_comment_diagnostics_and_typed_native_hole() {
    let arena = Allocator::default();
    let root = "前<template>{{ /*kept*/ value + }}</template>後";
    let start = root.find("{{").unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let owner = surface::parse_component_block(&arena, block);
    let syntax = owner.bindings()[0].syntax().unwrap();
    assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
    assert!(syntax.expression().is_none());
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap(),
        "/*kept*/"
    );
    assert!(syntax.diagnostics().count() > 0);
    for diagnostic in syntax.diagnostics() {
        for label in diagnostic.labels() {
            if let Ok(span) = label.authored_span() {
                assert!(span.start >= start as u32 && span.end <= end as u32);
                assert!(root.get(span.start as usize..span.end as usize).is_some());
            }
        }
    }
    assert_eq!(
        owner
            .text_for(owner.children().next().unwrap())
            .unwrap_err(),
        TextRefusal::NativeHole(EmbedHole::Syntax)
    );
}

#[test]
fn missing_close_retains_syntax_without_admission_or_a_fabricated_authored_tree() {
    let arena = Allocator::default();
    let owner = surface::parse_component_with_authored(&arena, "<p>{{ x }}").unwrap();
    assert_eq!(owner.bindings().len(), 1);
    assert!(owner.bindings()[0].syntax().unwrap().expression().is_some());
    let parent = owner.children().next().unwrap();
    assert_eq!(
        owner
            .text_for(parent.children().unwrap().next().unwrap())
            .unwrap_err(),
        TextRefusal::RecoveredComponent
    );
    assert!(owner.authored_children().is_none());
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}

#[test]
fn once_raw_historical_separators_and_every_pipe_are_explicit_original_refusals() {
    let arena = Allocator::default();
    for (source, kind) in [
        ("{{{ x }}}", TextBoundaryKind::RawInterpolation),
        ("{{{ x }}", TextBoundaryKind::RawInterpolation),
        ("{{* x }}", TextBoundaryKind::OneTimeInterpolation),
        ("{{&#42; x }}", TextBoundaryKind::OneTimeInterpolation),
        ("{{ a\r\n b }}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{&#13; x }}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ x &#8232;}}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ x \u{2029}}}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ value | upper 'arg' }}", TextBoundaryKind::PipeSyntax),
        ("{{ left || right }}", TextBoundaryKind::PipeSyntax),
        ("{{ 'a|b' }}", TextBoundaryKind::PipeSyntax),
        ("{{ left &#124; right }}", TextBoundaryKind::PipeSyntax),
        ("{{ }}", TextBoundaryKind::EmptyInterpolation),
    ] {
        let owner = surface::parse_component(&arena, source).unwrap();
        assert_eq!(owner.bindings()[0].boundary(), Some(kind), "{source}");
        assert!(owner.bindings()[0].syntax().is_none(), "{source}");
        assert_eq!(
            owner
                .text_for(owner.children().next().unwrap())
                .unwrap_err(),
            TextRefusal::Boundary(kind),
            "{source}"
        );
        assert_eq!(check_fidelity(owner.tree()), Ok(()));
    }
    // The historical once marker is tested before trim; leading space is not once.
    let owner = surface::parse_component(&arena, "{{ * x }}").unwrap();
    assert_eq!(owner.bindings()[0].boundary(), None);
    assert!(owner.bindings()[0].syntax().unwrap().hole().is_some());
}

#[test]
fn text_entities_decode_once_and_encoded_framing_stays_a_typed_refusal() {
    let arena = Allocator::default();
    for (source, expected) in [
        ("{{ '&#x96ea;' }}", "'雪'"),
        ("{{ '&amp;#42;' }}", "'&#42;'"),
        ("{{ &#32;value &amp;&amp; ok&#32; }}", "value && ok"),
    ] {
        let owner = surface::parse_component(&arena, source).unwrap();
        let binding = &owner.bindings()[0];
        assert_eq!(binding.source().unwrap().text(), expected);
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        assert!(core::ptr::eq(
            view.expression(),
            binding.syntax().unwrap().expression().unwrap()
        ));
    }
    for source in ["{{ '&#125;&#125;' }}", "{{ '&#123;' }}"] {
        let owner = surface::parse_component(&arena, source).unwrap();
        assert_eq!(
            owner.bindings()[0].boundary(),
            Some(TextBoundaryKind::EncodedDelimiter)
        );
        assert!(owner.bindings()[0].syntax().is_none());
    }
    let source = "&#123;&#123; x &#125;&#125; {{ y }}";
    let owner = surface::parse_component(&arena, source).unwrap();
    assert_eq!(owner.text_boundaries().len(), 4);
    for boundary in owner.text_boundaries() {
        assert_eq!(boundary.kind, TextBoundaryKind::EncodedDelimiter);
        assert!(matches!(boundary.span.slice(source), "&#123;" | "&#125;"));
    }
    assert_eq!(owner.bindings().len(), 1);
    let child = owner
        .children()
        .find(|child| matches!(child.surface(), SurfaceChild::Interpolation(_)))
        .unwrap();
    assert_eq!(
        owner.text_for(child).unwrap_err(),
        TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter)
    );
}

#[test]
fn literal_pre_uses_original_mode_and_does_not_observe_suppressed_callbacks() {
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, "<p v-pre>{{ x }}&#123;</p>{{ y }}").unwrap();
    assert_eq!(owner.bindings().len(), 1);
    assert!(owner.text_boundaries().is_empty());
    assert_eq!(owner.bindings()[0].source().unwrap().text(), "y");
    let mut children = owner.children();
    let parent = children.next().unwrap();
    assert!(matches!(
        parent.children().unwrap().next().unwrap().surface(),
        SurfaceChild::Text(_)
    ));
    assert_eq!(
        owner
            .text_for(children.next().unwrap())
            .unwrap()
            .binding()
            .source()
            .unwrap()
            .text(),
        "y"
    );
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}

#[test]
fn every_utf8_prefix_keeps_callback_spans_and_original_parser_root_in_bounds() {
    let arena = Allocator::default();
    let source = "前{{ /*雪*/ x &amp;&amp; 条件 }}{{{ raw }}}<p>{{ next }}</p>後";
    for end in 0..=source.len() {
        if !source.is_char_boundary(end) {
            continue;
        }
        let root = &source[..end];
        let owner = surface::parse_component_with_authored(&arena, root).unwrap();
        assert_eq!(check_fidelity(owner.tree()), Ok(()));
        for binding in owner.bindings() {
            assert_eq!(binding.content_span().slice(root), binding.raw_content());
            assert!(binding.span().end <= end as u32);
            if let Some(prepared) = binding.source() {
                assert_eq!(prepared.authored_root().as_ptr(), root.as_ptr());
                assert!(prepared.span().end <= end as u32);
                assert_eq!(
                    prepared.authored_span(Span::new(0, prepared.text().len() as u32)),
                    Ok(prepared.span())
                );
            }
        }
    }
}
