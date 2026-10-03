use super::{Allocator, Kind, SurfaceChild, selected};

#[test]
fn moved_owner_and_pending_growth_rejoin_exact_backing_without_second_iteration()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--前雪🌸--><template>first<!--middle-->  尾 \t </template>";
    let original = selected(&arena, source)?;
    let mut body = original.children();
    let first = body.next().ok_or("first")?;
    let receipt = original
        .prepare_condensed_root_text(first)
        .map_err(|_| "first receipt")?;
    let comment = body.next().ok_or("comment")?;
    assert!(matches!(comment.surface(), SurfaceChild::Comment(_)));
    let tail = body.next().ok_or("tail")?;
    let tail_receipt = original
        .prepare_condensed_root_text(tail)
        .map_err(|_| "tail receipt")?;
    assert_eq!(body.len(), 0);
    let mut parked = alloc::vec::Vec::new();
    parked.push(receipt);
    parked.push(tail_receipt);
    parked.reserve(32);
    let moved = core::hint::black_box(original);
    assert!(
        parked
            .first()
            .ok_or("first parked receipt")?
            .admitted_for_root_at(&moved, 0)
            .is_some()
    );
    assert!(
        parked
            .last()
            .ok_or("last parked receipt")?
            .admitted_for_root_at(&moved, 2)
            .is_some()
    );
    assert_eq!(parked.last().ok_or("tail")?.content(), Some(" 尾 "));
    assert!(
        parked
            .first()
            .ok_or("first")?
            .admitted_for_root_at(&moved, 2)
            .is_none()
    );
    assert!(
        parked
            .last()
            .ok_or("last")?
            .admitted_for_root_at(&moved, 0)
            .is_none()
    );
    assert!(
        parked
            .last()
            .ok_or("last")?
            .admitted_for_root_at(&moved, 3)
            .is_none()
    );
    assert_eq!(moved.children().len(), 3);
    crate::check_fidelity(&moved.component().carrier().tree).map_err(|_| "retained fidelity")?;
    Ok(())
}

#[test]
fn same_source_reparse_copied_source_and_sibling_slots_cannot_supply_foreign_origin()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>same<!--middle-->same</template>";
    let owner = selected(&arena, source)?;
    let child = owner.children().next().ok_or("original")?;
    let receipt = owner
        .prepare_condensed_root_text(child)
        .map_err(|_| "original receipt")?;
    assert!(receipt.admitted_for_root_at(&owner, 2).is_none());
    let copied = vize_l0::String::from(source);
    for foreign_source in [source, copied.as_str()] {
        let foreign = selected(&arena, foreign_source)?;
        assert!(receipt.admitted_for_root_at(&foreign, 0).is_none());
        assert!(matches!(
            owner.prepare_condensed_root_text(foreign.children().next().ok_or("foreign text")?),
            Err(Kind::ForeignComponent)
        ));
        assert!(
            foreign
                .prepare_condensed_root_text(foreign.children().next().ok_or("actual foreign")?)
                .is_ok()
        );
    }
    Ok(())
}

#[test]
fn nested_pre_rcdata_and_nontext_events_never_mint_root_whitespace_authority()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template><div> a \t b </div></template>",
        "<template><pre>\r\n a\t b\r\n</pre></template>",
        "<template><textarea> a\t b </textarea></template>",
        "<template><div v-pre>{{ a }} \t b </div></template>",
    ] {
        let owner = selected(&arena, source)?;
        let child = owner.children().next().ok_or("actual root element")?;
        assert!(matches!(
            owner.prepare_condensed_root_text(child.reborrow()),
            Err(Kind::NotText)
        ));
        let element = child.into_element().ok_or("actual original parent")?;
        let nested = element.children().next().ok_or("actual nested text")?;
        assert!(matches!(
            owner.prepare_condensed_root_text(nested),
            Err(Kind::NestedChild)
        ));
        crate::check_fidelity(&owner.component().carrier().tree)
            .map_err(|_| "original fidelity")?;
    }
    let owner = selected(&arena, "<template><!--comment-->{{ value }}</template>")?;
    for child in owner.children() {
        assert!(matches!(
            owner.prepare_condensed_root_text(child),
            Err(Kind::NotText)
        ));
    }
    Ok(())
}

#[test]
fn entity_spelling_refuses_before_decode_and_the_next_original_text_stays_available()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template>A&#32;&#9;B<!--separator-->next</template>",
        "<template>&#32;&#13;&#10;<!--separator-->next</template>",
        "<template>&amp;#32;<!--separator-->next</template>",
        "<template>literal&amp;<!--separator-->next</template>",
        "<template>literal&unknown;<!--separator-->next</template>",
    ] {
        let owner = selected(&arena, source)?;
        let mut body = owner.children();
        let first = body.next().ok_or("authored entity text")?;
        let original = first.surface();
        let before = arena.allocated_bytes();
        assert!(matches!(
            owner.prepare_condensed_root_text(first),
            Err(Kind::Entity)
        ));
        assert_eq!(arena.allocated_bytes(), before);
        assert!(core::ptr::eq(
            owner.children().next().ok_or("original remains")?.surface(),
            original
        ));
        assert!(matches!(
            body.next().ok_or("original comment")?.surface(),
            SurfaceChild::Comment(_)
        ));
        let receipt = owner
            .prepare_condensed_root_text(body.next().ok_or("actual next")?)
            .map_err(|_| "next original receipt")?;
        assert_eq!(receipt.content(), Some("next"));
        assert!(receipt.admitted_for_root_at(&owner, 2).is_some());
        crate::check_fidelity(&owner.component().carrier().tree).map_err(|_| "fidelity")?;
    }
    Ok(())
}

#[test]
fn original_unsupported_parser_observations_refuse_before_deriving_root_text()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>  original<div v-pre:[((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((key))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))]>body</div></template>";
    let owner = selected(&arena, source)?;
    assert_eq!(owner.component().carrier().unsupported.len(), 1);
    assert_eq!(
        owner.component().carrier().unsupported[0].error,
        crate::markup::DirectiveNameError::NestingLimit
    );
    let child = owner.children().next().ok_or("original text prefix")?;
    let original = child.surface();
    let before = arena.allocated_bytes();
    assert!(matches!(
        owner.prepare_condensed_root_text(child),
        Err(Kind::RecoveredComponent)
    ));
    assert_eq!(arena.allocated_bytes(), before);
    assert!(core::ptr::eq(
        owner
            .children()
            .next()
            .ok_or("original prefix retained")?
            .surface(),
        original
    ));
    assert!(!owner.component().carrier().unsupported.is_empty());
    crate::check_fidelity(&owner.component().carrier().tree)
        .map_err(|_| "original recovered fidelity")?;
    Ok(())
}

#[test]
fn source_selected_typescript_role_is_retained_without_a_caller_profile() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let source =
        "<!--前雪🌸--><template> a\t b </template><script setup lang=ts>const x=true</script>";
    let owner = selected(&arena, source)?;
    let receipt = owner
        .prepare_condensed_root_text(owner.children().next().ok_or("actual root")?)
        .map_err(|_| "original root text")?;
    assert_eq!(
        receipt.grammar(),
        crate::markup::NativeTemplateGrammar::TypeScriptModule
    );
    assert!(core::ptr::eq(receipt.block().root_source(), source));
    assert_eq!(receipt.span().slice(source), " a\t b ");
    assert_eq!(receipt.content(), Some(" a b "));
    assert!(receipt.admitted_for_root_at(&owner, 0).is_some());
    Ok(())
}
