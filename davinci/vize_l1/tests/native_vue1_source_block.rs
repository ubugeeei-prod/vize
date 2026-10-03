//! Complete-file custody laws for the original Vue 1 component construction.

use vize_l0::{Allocator, ErrorCode, SourceFrameError, SourceRoot, Span, cstr};
use vize_l1::dialect::LegacyVueVersion;
use vize_l1::dialect::vue1::surface::{self, SyntaxBoundary, SyntaxBoundaryKind};
use vize_l1::{SurfaceChild, check_fidelity};

fn contents<'a>(children: &[SurfaceChild<'a>]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for child in children {
        match child {
            SurfaceChild::Interpolation(node) => result.push(node.content.text),
            SurfaceChild::Element(node) => result.extend(contents(&node.children)),
            _ => {}
        }
    }
    result
}

#[test]
fn standalone_apis_retain_the_original_base_zero_root_and_error_spelling() {
    let arena = Allocator::default();
    let source = String::from("前 {{{ 雪 }}} 後");
    let normal = surface::parse_component(&arena, &source).unwrap();
    let authored = surface::parse_component_with_authored(&arena, &source).unwrap();
    for parsed in [normal, authored] {
        assert_eq!(parsed.version(), LegacyVueVersion::V1);
        assert_eq!(parsed.block().span(), Span::new(0, source.len() as u32));
        assert_eq!(parsed.block().root_source().as_ptr(), source.as_ptr());
        assert_eq!(parsed.block().source().as_ptr(), source.as_ptr());
        assert_eq!(parsed.tree().source.as_ptr(), source.as_ptr());
        assert_eq!(check_fidelity(parsed.tree()), Ok(()));
        assert!(parsed.errors().is_empty());
        assert!(parsed.unsupported().is_empty());
    }
    let legacy_spelling = surface::SourceError::SourceTooLarge;
    let same_source_error: SourceFrameError = legacy_spelling;
    assert_eq!(same_source_error, SourceFrameError::SourceTooLarge);
}

#[test]
fn nonzero_unicode_block_keeps_original_raw_tokens_attribute_and_root_identity() {
    let arena = Allocator::default();
    let source = "前🍣<template><p title='雪'>{{{ 雪 }}}{{ next }}</p></template>後";
    let start = source.find("<p").unwrap();
    let end = source.find("</template>").unwrap();
    let text = source.get(start..end).unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(text, start as u32)
        .unwrap();
    let parsed = surface::parse_component_block(&arena, block);
    assert_eq!(parsed.block(), block);
    assert_eq!(parsed.block().root_source().as_ptr(), source.as_ptr());
    assert_eq!(parsed.tree().source.as_ptr(), text.as_ptr());
    assert_eq!(parsed.version(), LegacyVueVersion::V1);
    assert_eq!(check_fidelity(parsed.tree()), Ok(()));
    assert!(parsed.errors().is_empty());
    assert!(parsed.unsupported().is_empty());
    let SurfaceChild::Element(p) = parsed.tree().children.first().unwrap() else {
        panic!("original p")
    };
    assert_eq!(p.open.lt_name.text, "<p");
    assert_eq!(block.offset_of(p.open.lt_name.text), Some(start as u32));
    let title = p
        .open
        .attrs
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .content
        .text;
    assert_eq!(title, "雪");
    assert_eq!(
        block.span_of(title),
        Some(Span::new((start + 10) as u32, (start + 13) as u32))
    );
    let raw_start = source.find("{{{").unwrap();
    let SurfaceChild::Interpolation(raw) = p.children.first().unwrap() else {
        panic!("original raw interpolation")
    };
    for (token, value, relative) in [
        (raw.open, "{{{", 0),
        (raw.content, " 雪 ", 3),
        (raw.close, "}}}", 8),
    ] {
        assert_eq!(token.text, value);
        assert_eq!(
            token.text.as_ptr(),
            source.as_ptr().wrapping_add(raw_start + relative)
        );
        assert_eq!(
            block.offset_of(token.text),
            Some((raw_start + relative) as u32)
        );
    }
    assert!(raw.is_raw_html());
    assert_eq!(contents(&p.children), [" 雪 ", " next "]);
    let foreign = String::from("雪");
    assert_eq!(block.span_of(&foreign), None);
}

#[test]
fn every_existing_syntax_boundary_uses_complete_authored_file_coordinates() {
    let arena = Allocator::default();
    let source = "前<template>{{{ x }} {{}} {{* x}} {{ a\r b }}</template>後";
    let start = source.find("{{{").unwrap();
    let end = source.find("</template>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let parsed = surface::parse_component_block(&arena, block);
    assert_eq!(check_fidelity(parsed.tree()), Ok(()));
    let expected = [
        ("{{{ x }}", SyntaxBoundaryKind::RawDelimiterRecovery),
        ("{{}}", SyntaxBoundaryKind::EmptyInterpolation),
        ("{{* x}}", SyntaxBoundaryKind::OneTimeInterpolation),
        ("{{ a\r b }}", SyntaxBoundaryKind::HistoricalLineSeparator),
    ]
    .map(|(text, kind)| {
        let start = source.find(text).unwrap() as u32;
        SyntaxBoundary {
            span: Span::new(start, start + text.len() as u32),
            kind,
        }
    });
    assert_eq!(parsed.unsupported(), expected);
    for boundary in parsed.unsupported() {
        assert!(block.contains_block_span(boundary.span));
    }
}

#[test]
fn original_markup_diagnostics_stop_at_the_selected_block_in_both_projections() {
    for (input, code, relative) in [
        ("<div", ErrorCode::EofInTag, 4),
        ("{{ 未完", ErrorCode::MissingInterpolationEnd, 9),
        ("<!-- 未完", ErrorCode::EofInComment, 0),
    ] {
        let arena = Allocator::default();
        let source = cstr!("前🍣<template>{input}</template><p>後</p>");
        let start = source.find(input).unwrap();
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(
                source.get(start..start + input.len()).unwrap(),
                start as u32,
            )
            .unwrap();
        for parsed in [
            surface::parse_component_block(&arena, block),
            surface::parse_component_with_authored_block(&arena, block),
        ] {
            assert_eq!(check_fidelity(parsed.tree()), Ok(()));
            assert_eq!(parsed.tree().source, input);
            assert_eq!(
                parsed
                    .errors()
                    .iter()
                    .map(|error| (error.code, error.offset))
                    .collect::<Vec<_>>(),
                [(code, start as u32 + relative)]
            );
            assert!(parsed.unsupported().is_empty());
        }
    }
}

#[test]
fn normal_and_authored_recovery_keep_the_same_original_block_and_text_pointers() {
    let arena = Allocator::default();
    let input = "<a v-pre><span>{{{before}}}<a>{{{after}}}</a>{{{tail}}}</span></a>";
    let source = cstr!("前🍣<template>{input}</template>後");
    let start = source.find(input).unwrap();
    let block = SourceRoot::new(&source)
        .unwrap()
        .block(
            source.get(start..start + input.len()).unwrap(),
            start as u32,
        )
        .unwrap();
    let parsed = surface::parse_component_with_authored_block(&arena, block);
    assert_eq!(parsed.block(), block);
    let authored = parsed.authored().unwrap();
    for tree in [parsed.tree(), authored] {
        assert_eq!(tree.source.as_ptr(), block.source().as_ptr());
        assert_eq!(check_fidelity(tree), Ok(()));
        assert_eq!(contents(&tree.children), ["after", "tail"]);
        for text in contents(&tree.children) {
            let expected = source.find(text).unwrap();
            assert_eq!(text.as_ptr(), source.as_ptr().wrapping_add(expected));
            assert_eq!(block.offset_of(text), Some(expected as u32));
        }
    }
}

#[test]
fn identical_bytes_from_a_foreign_root_or_wrong_occurrence_cannot_supply_the_frame() {
    let arena = Allocator::default();
    let source = String::from("前{{{ 雪 }}}後{{{ 雪 }}}");
    let foreign = String::from("{{{ 雪 }}}");
    let start = source.find("{{{").unwrap();
    let second = source.rfind("{{{").unwrap();
    let root = SourceRoot::new(&source).unwrap();
    let first_text = source.get(start..start + foreign.len()).unwrap();
    let second_text = source.get(second..).unwrap();
    assert_eq!(
        root.block(&foreign, start as u32),
        Err(SourceFrameError::BlockNotRootSlice)
    );
    assert_eq!(
        root.block(second_text, start as u32),
        Err(SourceFrameError::BlockNotRootSlice)
    );
    for (text, offset) in [(first_text, start), (second_text, second)] {
        let block = root.block(text, offset as u32).unwrap();
        let parsed = surface::parse_component_block(&arena, block);
        let pieces = contents(&parsed.tree().children);
        let text = *pieces.first().unwrap();
        assert_eq!(text, " 雪 ");
        assert_eq!(block.offset_of(text), Some((offset + 3) as u32));
        assert_eq!(parsed.block().root_source().as_ptr(), source.as_ptr());
    }
}

#[test]
fn every_unicode_recovery_prefix_keeps_its_own_complete_root_and_bounded_observations() {
    let input = "中🍣<div v-pre.foo>{{{未完}} <span/>{{{次}}}</div><p>{{最後}}</p>";
    for end in input
        .char_indices()
        .map(|(index, _)| index)
        .chain([input.len()])
    {
        let arena = Allocator::default();
        let prefix = input.get(..end).unwrap();
        let source = cstr!("前<script>const 雪=1</script><template>{prefix}</template>後");
        let start = source.find("<template>").unwrap() + "<template>".len();
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(source.get(start..start + end).unwrap(), start as u32)
            .unwrap();
        let parsed = surface::parse_component_with_authored_block(&arena, block);
        assert_eq!(parsed.version(), LegacyVueVersion::V1);
        assert_eq!(parsed.block().root_source().as_ptr(), source.as_ptr());
        assert_eq!(parsed.tree().source.as_ptr(), block.source().as_ptr());
        assert_eq!(check_fidelity(parsed.tree()), Ok(()), "{end}");
        if let Some(authored) = parsed.authored() {
            assert_eq!(authored.source.as_ptr(), block.source().as_ptr());
            assert_eq!(check_fidelity(authored), Ok(()), "{end}");
        }
        for error in parsed.errors() {
            assert!(
                block.contains_block_span(Span::new(error.offset, error.offset)),
                "{end}"
            );
        }
        for boundary in parsed.unsupported() {
            assert!(block.contains_block_span(boundary.span), "{end}");
        }
    }
}
