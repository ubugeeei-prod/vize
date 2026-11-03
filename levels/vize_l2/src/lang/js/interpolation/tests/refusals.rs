use super::{
    Allocator, NativeInterpolationInput, NativeInterpolationInputError, Test, check, same, selected,
};

#[test]
fn equal_text_foreign_owner_and_duplicate_original_child_cannot_join() -> Test {
    let arena = Allocator::default();
    let source = "<template>{{ 1 }}{{ 1 }}</template>";
    let owner = selected(&arena, source)?;
    let foreign = selected(&arena, source)?;
    let operand = owner
        .observe_interpolation_expression(owner.children().next().ok_or("child missing")?)
        .map_err(|_| "observation refused")?;
    let mut input = NativeInterpolationInput::from_operand(operand);
    let ast = input.operand().syntax().expression().ok_or("AST missing")?;
    let fail = input.admitted_for(
        &foreign,
        foreign.children().next().ok_or("foreign child missing")?,
    );
    check(
        matches!(fail, Err(NativeInterpolationInputError::UnadmittedOriginal)),
        "equal foreign owner admitted",
    )?;
    check(
        input.expression.is_none(),
        "foreign join transferred metadata",
    )?;
    let duplicate = owner.children().nth(1).ok_or("duplicate child missing")?;
    check(
        matches!(
            input.admitted_for(&owner, duplicate),
            Err(NativeInterpolationInputError::UnadmittedOriginal)
        ),
        "same-text different ordinal admitted",
    )?;
    check(
        input.expression.is_none(),
        "wrong ordinal transferred metadata",
    )?;
    let expression = input
        .admitted_for(&owner, owner.children().next().ok_or("original missing")?)
        .map_err(|_| "original join refused")?
        .expression();
    check(core::ptr::eq(expression.ast, ast), "AST lost after refusal")?;
    check(
        matches!(
            input.admitted_for(
                &foreign,
                foreign.children().next().ok_or("foreign missing")?
            ),
            Err(NativeInterpolationInputError::UnadmittedOriginal)
        ),
        "cached metadata admitted foreign owner",
    )?;
    check(
        core::ptr::eq(input.expression.ok_or("cached metadata lost")?, expression),
        "failed rejoin changed cache",
    )?;
    same(
        input.operand().full_span().slice(source),
        "{{ 1 }}",
        "failed join changed full span",
    )
}

#[test]
fn same_ordinal_under_another_original_parent_is_refused() -> Test {
    let arena = Allocator::default();
    let source = "<template><p>{{ 1 }}</p><p>{{ 1 }}</p></template>";
    let owner = selected(&arena, source)?;
    let first = owner
        .children()
        .next()
        .ok_or("first missing")?
        .into_element()
        .ok_or("not element")?;
    let second = owner
        .children()
        .nth(1)
        .ok_or("second missing")?
        .into_element()
        .ok_or("not element")?;
    let operand = owner
        .observe_interpolation_expression(first.children().next().ok_or("first child missing")?)
        .map_err(|_| "observation refused")?;
    let mut input = NativeInterpolationInput::from_operand(operand);
    check(
        matches!(
            input.admitted_for(
                &owner,
                second.children().next().ok_or("second child missing")?
            ),
            Err(NativeInterpolationInputError::UnadmittedOriginal)
        ),
        "foreign parent admitted",
    )?;
    check(
        input.expression.is_none(),
        "foreign parent transferred metadata",
    )?;
    let view = input
        .admitted_for(
            &owner,
            first.children().next().ok_or("original child missing")?,
        )
        .map_err(|_| "original parent join refused")?;
    check(
        core::ptr::eq(
            view.original()
                .child()
                .parent_element()
                .ok_or("parent lost")?,
            first.surface(),
        ),
        "original parent lost",
    )
}

#[test]
fn typed_holes_keep_original_full_syntax_and_stock_diagnostics() -> Test {
    for source in [
        "<template>{{ }}</template>",
        "<template>{{ value + }}</template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source)?;
        let operand = owner
            .observe_interpolation_expression(owner.children().next().ok_or("child missing")?)
            .map_err(|_| "hole observation refused")?;
        let hole = operand.syntax().hole().ok_or("fixture hole missing")?;
        let raw = operand.raw_content();
        let full = operand.full_span();
        let diagnostics = operand.syntax().diagnostics().count();
        let mut input = NativeInterpolationInput::from_operand(operand);
        check(
            matches!(input.admitted_for(&owner, owner.children().next().ok_or("original missing")?), Err(NativeInterpolationInputError::Hole(actual)) if actual == hole),
            "typed hole admitted or changed",
        )?;
        check(
            input.expression.is_none(),
            "hole fabricated neutral expression",
        )?;
        same(
            input.operand().syntax().hole(),
            Some(hole),
            "stock hole lost",
        )?;
        same(
            input.operand().syntax().diagnostics().count(),
            diagnostics,
            "stock diagnostics lost",
        )?;
        same(input.operand().raw_content(), raw, "hole full raw content")?;
        same(input.operand().full_span(), full, "hole full delimiters")?;
        check(
            core::ptr::eq(input.operand().syntax().source().authored_root(), source),
            "hole original source lost",
        )?;
    }
    Ok(())
}
