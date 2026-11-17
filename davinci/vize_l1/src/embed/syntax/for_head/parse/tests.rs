use oxc_ast::ast::Expression;
use oxc_parser::ParseOptions;
use vize_l0::{Allocator, SourceFrameError, SourceRoot, Span};

use super::{NativeForHead, NativeForInput, NativeForInputError, NativeForRefusal};
use crate::embed::{Embed, EmbedSource, Grammar, Lang, Shape};

mod refused;

fn observe<'a>(arena: &'a Allocator, text: &'a str, lang: Lang) -> NativeForHead<'a> {
    NativeForInput::attribute_in(
        arena,
        SourceRoot::new(text).unwrap().whole_block(),
        text,
        lang,
    )
    .unwrap()
    .observe()
}

#[test]
fn intrinsic_head_keeps_actual_unicode_block_value_two_roots_and_profiles() {
    let arena = Allocator::default();
    let file = "頭<div v-for='α in β'></div>尾";
    let start = file.find("α in β").unwrap();
    let value = file.get(start..start + "α in β".len()).unwrap();
    let root = SourceRoot::new(file).unwrap();
    let block = root.block(file.get(3..file.len() - 3).unwrap(), 3).unwrap();
    for lang in [Lang::Js, Lang::Ts] {
        let head = NativeForInput::attribute_in(&arena, block, value, lang)
            .unwrap()
            .observe();
        let alias = head.aliases().unwrap().unwrap();
        let collection = head.collection().unwrap().unwrap();
        let formals = alias.parameters().unwrap().as_ptr();
        let expression = collection.expression().unwrap() as *const _;
        let admitted = head.admitted_dense().unwrap();
        assert_eq!(admitted.source_block(), block);
        assert_eq!(admitted.authored_value().as_ptr(), value.as_ptr());
        assert_eq!(
            admitted.authored_value_span(),
            Span::new(start as u32, (start + value.len()) as u32)
        );
        assert_eq!(admitted.source().text().as_ptr(), value.as_ptr());
        assert_eq!(admitted.aliases().parameters().items.as_ptr(), formals);
        assert_eq!(admitted.collection().expression() as *const _, expression);
        for profile in [
            admitted.aliases().source_type(),
            admitted.collection().source_type(),
        ] {
            assert!(profile.is_module());
            assert!(!profile.is_jsx() && !profile.is_typescript_definition());
            assert_eq!(profile.is_typescript(), lang == Lang::Ts);
        }
        assert_eq!(admitted.aliases().options(), ParseOptions::default());
        assert_eq!(admitted.collection().options(), ParseOptions::default());
        assert_eq!(admitted.alias_source().text(), "α");
        assert_eq!(admitted.collection_source().text(), "β");
        assert_eq!(head.diagnostics().count(), 0);
    }
}

#[test]
fn selected_alias_window_uses_the_original_once_parenthesis_split() {
    let arena = Allocator::default();
    for (text, aliases) in [
        ("item in items", "item"),
        ("( item, index ) of items", "item, index"),
    ] {
        let head = observe(&arena, text, Lang::Js);
        let admitted = head.admitted_dense().unwrap();
        assert_eq!(admitted.alias_source().text(), aliases);
        assert_eq!(
            admitted.aliases().content().as_ptr(),
            admitted.alias_source().text().as_ptr()
        );
        assert_eq!(
            admitted.collection().content().as_ptr(),
            admitted.collection_source().text().as_ptr()
        );
        let begin = text.find(aliases).unwrap();
        assert_eq!(
            admitted.alias_source().span(),
            Span::new(begin as u32, (begin + aliases.len()) as u32)
        );
        assert_eq!(admitted.authored_value(), text);
    }
}

#[test]
fn foreign_equal_bytes_and_outside_block_values_keep_the_original_rejection() {
    let arena = Allocator::default();
    let text = "item in items|other in values";
    let value = text.get(..13).unwrap();
    let root = SourceRoot::new(text).unwrap();
    let block = root.block(value, 0).unwrap();
    let equal = vize_l0::String::from(value);
    let outside = text.get(14..).unwrap();
    let bytes = arena.allocated_bytes();
    for raw in [equal.as_str(), outside] {
        let error = NativeForInput::attribute_in(&arena, block, raw, Lang::Ts)
            .err()
            .unwrap();
        assert_eq!(error.error(), NativeForInputError::ForeignValue);
        assert_eq!(error.source_block(), block);
        assert_eq!(error.authored_value().as_ptr(), raw.as_ptr());
        assert_eq!(error.authored_value().len(), raw.len());
        assert_eq!(error.lang(), Lang::Ts);
    }
    assert_eq!(arena.allocated_bytes(), bytes);
    let unicode = SourceRoot::new("éitem in items").unwrap();
    assert_eq!(unicode.block("x", 1), Err(SourceFrameError::BlockBoundary));
}

#[test]
fn valid_partial_extent_is_only_local_source_proof_not_an_attribute_receipt() {
    let arena = Allocator::default();
    let full = "item in items trailing";
    let partial = full.get(..13).unwrap();
    let block = SourceRoot::new(full).unwrap().whole_block();
    let partial_head = NativeForInput::attribute_in(&arena, block, partial, Lang::Js)
        .unwrap()
        .observe();
    let admitted = partial_head.admitted_dense().unwrap();
    assert_eq!(admitted.authored_value_span(), Span::new(0, 13));
    assert_eq!(admitted.authored_value().as_ptr(), full.as_ptr());
    assert_ne!(admitted.authored_value().len(), full.len());
    let complete = NativeForInput::attribute_in(&arena, block, full, Lang::Js)
        .unwrap()
        .observe();
    assert!(complete.admitted_dense().is_none());
    assert!(complete.diagnostics().count() > 0);
    // Complete original Attribute pointer/length/span admission belongs to the
    // real upper receiver. This public SourceBlock constructor cannot infer it.
}

#[test]
fn raw_dense_and_duplicate_names_do_not_mint_missing_origin_or_semantic_proofs() {
    let arena = Allocator::default();
    let text = "item in items";
    let raw = super::super::parse_vue_for_head_once(
        &arena,
        Embed {
            grammar: Grammar {
                lang: Lang::Js,
                shape: Shape::ForHead,
            },
            source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
        },
    );
    assert!(raw.dense().is_some());
    assert_eq!(
        raw.native_refusal(),
        Some(NativeForRefusal::UnpreparedSource)
    );
    assert!(raw.admitted_dense().is_none());
    let duplicate = observe(&arena, "(item,item) in items", Lang::Js);
    assert_eq!(
        duplicate
            .admitted_dense()
            .unwrap()
            .aliases()
            .parameters()
            .items
            .len(),
        2
    );
    // Canonical construction still owns its existing once binding enumeration
    // and duplicate-name preflight; no generic uniqueness is claimed here.
}

#[test]
fn short_joint_borrow_ends_before_retaining_the_complete_normal_owner() {
    let arena = Allocator::default();
    let head = observe(&arena, "item in items", Lang::Js);
    let root = {
        let admitted = head.admitted_dense().unwrap();
        let Expression::Identifier(identifier) = admitted.collection().expression() else {
            panic!("direct identifier")
        };
        core::ptr::from_ref(&**identifier)
    };
    let kept = alloc::vec![head];
    let Expression::Identifier(identifier) =
        kept[0].collection().unwrap().unwrap().expression().unwrap()
    else {
        panic!("original identifier")
    };
    assert_eq!(core::ptr::from_ref(&**identifier), root);
    assert_eq!(kept[0].source().text(), "item in items");
}
