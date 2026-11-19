use super::{Allocator, NativeInterpolationInput, Test, check, same, selected};
use oxc_span::GetSpan;
use vize_l1::embed::Lang;

#[test]
fn root_original_stock_ast_and_coordinates_survive_moves_and_cached_rejoins() -> Test {
    let arena = Allocator::default();
    let source = "<!--前--><template>{{ \t/*kept*/ '雪&amp;🌸' \n}}<!--tail--></template>";
    let owner = selected(&arena, source)?;
    let mut children = owner.children();
    let child = children.next().ok_or("original child missing")?;
    let surface = child.surface();
    let operand = owner
        .observe_interpolation_expression(child.reborrow())
        .map_err(|_| "original observation refused")?;
    let ast = operand
        .syntax()
        .expression()
        .ok_or("original AST missing")?;
    let decoded = operand.syntax().source().text();
    let mut input = NativeInterpolationInput::from_operand(operand);
    let expression = {
        let view = input
            .admitted_for(&owner, child)
            .map_err(|_| "original join refused")?;
        check(
            core::ptr::eq(
                view.input()
                    .operand()
                    .syntax()
                    .expression()
                    .ok_or("AST missing")?,
                ast,
            ),
            "stock AST changed",
        )?;
        check(
            core::ptr::eq(view.original().child().surface(), surface),
            "surface changed",
        )?;
        check(
            view.original().child().parent_element().is_none(),
            "root gained parent",
        )?;
        same(view.original().child().ordinal(), 0, "root ordinal")?;
        check(
            core::ptr::eq(view.expression().ast, ast),
            "neutral AST copied",
        )?;
        check(
            core::ptr::eq(view.expression().source, decoded),
            "decoded text copied",
        )?;
        check(
            view.expression().coordinates.is_some(),
            "coordinates missing",
        )?;
        check(
            view.expression().matches_authored_source(source),
            "whole authored source lost",
        )?;
        let authored = view
            .expression()
            .authored_span(
                view.expression()
                    .ast_span_to_source(ast.span())
                    .ok_or("decoded span missing")?,
            )
            .ok_or("authored span missing")?;
        same(
            authored.slice(source),
            "'雪&amp;🌸'",
            "authored entity projection",
        )?;
        view.expression()
    };
    let coordinates = expression
        .coordinates
        .ok_or("coordinate metadata missing")?;
    let mut parked = alloc::vec::Vec::new();
    parked.push(input);
    parked.reserve(32);
    let input = parked.first_mut().ok_or("moved input missing")?;
    let view = input
        .admitted_for(&owner, owner.children().next().ok_or("same child missing")?)
        .map_err(|_| "rejoin refused")?;
    check(
        core::ptr::eq(view.expression(), expression),
        "metadata was transferred again",
    )?;
    check(
        core::ptr::eq(
            view.expression()
                .coordinates
                .ok_or("cached coordinates missing")?,
            coordinates,
        ),
        "coordinates copied again",
    )?;
    same(
        view.input().operand().full_span().slice(source),
        "{{ \t/*kept*/ '雪&amp;🌸' \n}}",
        "complete delimiters",
    )?;
    same(
        view.input().operand().content_span().slice(source),
        view.input().operand().raw_content(),
        "full authored content",
    )?;
    same(
        view.input().operand().syntax().comments().count(),
        1,
        "stock comments",
    )?;
    same(
        view.input().operand().syntax().diagnostics().count(),
        0,
        "stock diagnostics",
    )?;
    same(
        view.input().operand().syntax().grammar().lang,
        Lang::Js,
        "intrinsic JS grammar",
    )?;
    same(children.len(), 1, "join consumed root iterator")
}

#[test]
fn nested_ts_original_parent_ordinal_and_stock_observation_stay_owned() -> Test {
    let arena = Allocator::default();
    let source = "<template><p>prefix{{ /*kept*/ value as boolean }}</p></template><script setup lang=ts>const value=true</script>";
    let owner = selected(&arena, source)?;
    let parent = owner
        .children()
        .next()
        .ok_or("parent missing")?
        .into_element()
        .ok_or("not element")?;
    let mut children = parent.children();
    let _ = children.next().ok_or("prefix missing")?;
    let child = children.next().ok_or("interpolation missing")?;
    let operand = owner
        .observe_interpolation_expression(child.reborrow())
        .map_err(|_| "observation refused")?;
    let ast = operand.syntax().expression().ok_or("AST missing")?;
    let mut input = NativeInterpolationInput::from_operand(operand);
    let view = input
        .admitted_for(&owner, child)
        .map_err(|_| "nested join refused")?;
    check(
        core::ptr::eq(
            view.original()
                .child()
                .parent_element()
                .ok_or("parent identity missing")?,
            parent.surface(),
        ),
        "parent changed",
    )?;
    same(view.original().child().ordinal(), 1, "nested ordinal")?;
    check(core::ptr::eq(view.expression().ast, ast), "TS AST changed")?;
    same(
        view.input().operand().syntax().grammar().lang,
        Lang::Ts,
        "intrinsic TS grammar",
    )?;
    check(
        view.input()
            .operand()
            .syntax()
            .source_type()
            .is_typescript(),
        "TS parser role lost",
    )?;
    check(
        view.input().operand().syntax().source_type().is_module(),
        "Module parser role lost",
    )?;
    same(
        view.input().operand().syntax().comments().count(),
        1,
        "TS stock comment",
    )?;
    same(
        view.input().operand().raw_content(),
        " /*kept*/ value as boolean ",
        "full TS raw content",
    )?;
    same(children.len(), 0, "nested iterator was rescanned")
}
