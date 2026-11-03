//! Existing original lexer facts remain attached to genuine body operands.

use super::{Allocator, selected};
use crate::embed::{Lang, syntax::EmbedHole};
use crate::markup::NativeInterpolationOperand;

fn operand<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeInterpolationOperand<'a>, &'static str> {
    let owner = selected(arena, source)?;
    let child = owner.children().next().ok_or("missing original root")?;
    owner
        .observe_interpolation_expression(child)
        .map_err(|_| "original observation failed")
}

#[test]
fn original_expression_projection_retains_decoded_legacy_literal_facts() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    for source in [
        "<template>{{ /*keep*/ 010 }}</template>",
        "<template>{{ /*keep*/ 08 }}</template>",
        "<template>{{ /*keep*/ 09.5 }}</template>",
        "<template>{{ /*keep*/ 08e1 }}</template>",
        r"<template>{{ /*keep*/ '\1' }}</template>",
        r"<template>{{ /*keep*/ '\8' }}</template>",
        r"<template>{{ /*keep*/ '\9' }}</template>",
        r"<template>{{ /*keep*/ '\00' }}</template>",
    ] {
        let original = operand(&arena, source)?;
        let admitted = original
            .syntax()
            .admitted_expression()
            .ok_or("syntax refused")?;
        assert_eq!(admitted.has_legacy_literals(), true, "{source}");
        assert_eq!(admitted.source_type().is_module(), true);
        assert_eq!(original.syntax().grammar().lang, Lang::Js);
        assert_eq!(original.syntax().diagnostics().count(), 0);
        assert_eq!(original.syntax().comments().count(), 1);
        assert_eq!(
            original.content_span().slice(source),
            original.raw_content()
        );
        assert_eq!(
            core::ptr::eq(original.syntax().source().authored_root(), source),
            true
        );
    }
    Ok(())
}

#[test]
fn comments_raw_escapes_and_modern_literals_cannot_forge_legacy_decoding()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template>{{ 0 }}</template>",
        "<template>{{ 0o10 }}</template>",
        "<template>{{ 0x10 }}</template>",
        "<template>{{ 0b10 }}</template>",
        "<template>{{ 0.1 }}</template>",
        "<template>{{ 0e1 }}</template>",
        "<template>{{ '雪🌸' }}</template>",
        r"<template>{{ '\0' }}</template>",
        r"<template>{{ '\x01' }}</template>",
        r"<template>{{ '\u0001' }}</template>",
        r"<template>{{ '\\1' }}</template>",
        r"<template>{{ '\🌸' }}</template>",
        r"<template>{{ /*010 '\1'*/ 1 }}</template>",
        r"<template>{{ String.raw`\1` }}</template>",
    ] {
        let original = operand(&arena, source)?;
        let admitted = original
            .syntax()
            .admitted_expression()
            .ok_or("syntax refused")?;
        assert_eq!(admitted.has_legacy_literals(), false, "{source}");
        assert_eq!(original.syntax().diagnostics().count(), 0);
        assert_eq!(original.syntax().hole(), None);
    }
    Ok(())
}

#[test]
fn original_entity_decode_ts_grammar_and_operand_move_keep_the_same_receipt()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (source, decoded, legacy) in [
        (
            "<template>{{ /*keep*/ &#48;10 }}</template><script setup lang=ts>const unused=1</script>",
            "/*keep*/ 010",
            true,
        ),
        (
            r"<template>{{ /*keep*/ '&#92;8' }}</template><script setup lang=ts>const unused=1</script>",
            r"/*keep*/ '\8'",
            true,
        ),
        (
            "<template>{{ /*keep*/ 0o10 }}</template><script setup lang=ts>const unused=1</script>",
            "/*keep*/ 0o10",
            false,
        ),
    ] {
        let owner = selected(&arena, source)?;
        let child = owner.children().next().ok_or("missing original root")?;
        let original = owner
            .observe_interpolation_expression(child.reborrow())
            .map_err(|_| "original observation failed")?;
        let expression = original
            .syntax()
            .admitted_expression()
            .ok_or("syntax refused")?;
        let ast = core::ptr::from_ref(expression.expression());
        let comment = original
            .syntax()
            .comments()
            .next()
            .ok_or("missing original comment")?
            .text()
            .map_err(|_| "comment projection failed")?
            .as_ptr();
        assert_eq!(original.syntax().source().text(), decoded);
        assert_eq!(expression.has_legacy_literals(), legacy);
        let moved = core::hint::black_box(original);
        let view = moved
            .admitted_for(&owner, child)
            .ok_or("original join refused")?;
        let joined = view.expression().ok_or("joined expression refused")?;
        assert_eq!(joined.has_legacy_literals(), legacy);
        assert_eq!(core::ptr::from_ref(joined.expression()), ast);
        assert_eq!(view.operand().syntax().grammar().lang, Lang::Ts);
        assert_eq!(view.operand().syntax().diagnostics().count(), 0);
        assert_eq!(
            view.operand()
                .syntax()
                .comments()
                .next()
                .ok_or("missing moved comment")?
                .text()
                .map_err(|_| "moved comment projection failed")?
                .as_ptr(),
            comment
        );
        assert_eq!(view.child().ordinal(), 0);
        assert_eq!(view.child().parent_element().is_none(), true);
    }
    Ok(())
}

#[test]
fn original_syntax_holes_still_cannot_supply_an_admitted_literal_receipt()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>{{ /*keep*/ 010 + }}</template>";
    let original = operand(&arena, source)?;
    assert_eq!(original.syntax().hole(), Some(EmbedHole::Syntax));
    assert_eq!(original.syntax().admitted_expression().is_none(), true);
    assert_eq!(original.syntax().diagnostics().count() > 0, true);
    assert_eq!(original.syntax().comments().count(), 1);
    assert_eq!(original.raw_content(), " /*keep*/ 010 + ");
    Ok(())
}
