//! Generic completed-branch extraction laws, not Native source admission.
//! Retained expressions use the existing real JS load-path constructor.

use vize_l0::{Allocator, Box, Span, String, Vec};
use vize_l2::analysis::if_facts::{BranchKey, BranchKeyKind, take_carrier_key};
use vize_l2::dump::{Op as DumpOp, Page};
use vize_l2::expr::{ExprRef, JsExpr};
use vize_l2::op::{
    Attribute, BindOp, BindingOp, ComponentOp, DynamicName, ElementOp, IfBranch, IfOp, Namespace,
    Op, Region, SlotOp,
};

type Check<T> = Result<T, String>;

fn span(source: &str, needle: &str) -> Check<Span> {
    let start = source
        .find(needle)
        .ok_or_else(|| String::from("missing fixture token"))?;
    let start = u32::try_from(start).map_err(|_| String::from("fixture start"))?;
    let len = u32::try_from(needle.len()).map_err(|_| String::from("fixture length"))?;
    Ok(Span::new(start, start + len))
}

fn js<'a>(arena: &'a Allocator, source: &'a str, name: &'a str) -> Check<ExprRef<'a>> {
    let expression = JsExpr::parse_in(arena, name, span(source, name)?)
        .map_err(|_| String::from("fixture expression must really parse"))?;
    Ok(ExprRef::Js(expression))
}

#[derive(Clone, Copy)]
enum Carrier {
    Element,
    Component,
    Slot,
}

fn carrier<'a>(
    arena: &'a Allocator,
    kind: Carrier,
    attributes: Vec<'a, Attribute<'a>>,
    bindings: Vec<'a, BindingOp<'a>>,
    span: Span,
) -> Op<'a> {
    let children = Region {
        ops: Vec::new_in(&arena),
    };
    match kind {
        Carrier::Element => Op::Element(Box::new_in(
            ElementOp {
                tag: "p",
                namespace: Namespace::Html,
                attributes,
                bindings,
                children,
                span,
            },
            &arena,
        )),
        Carrier::Component => Op::Component(Box::new_in(
            ComponentOp {
                name: "Panel",
                attributes,
                bindings,
                children,
                span,
            },
            &arena,
        )),
        Carrier::Slot => Op::Slot(Box::new_in(
            SlotOp {
                name: DynamicName::Static("default"),
                attributes,
                bindings,
                fallback: children,
                span,
            },
            &arena,
        )),
    }
}

fn root<'a>(arena: &'a Allocator, ops: Vec<'a, Op<'a>>, span: Span) -> Vec<'a, Op<'a>> {
    Vec::from_iter_in(
        [Op::If(Box::new_in(
            IfOp {
                branches: Vec::from_iter_in(
                    [IfBranch {
                        condition: None,
                        region: Region { ops },
                        span,
                    }],
                    &arena,
                ),
                span,
            },
            &arena,
        ))],
        &arena,
    )
}

fn take(root: &mut [Op<'_>]) -> Check<Option<BranchKey>> {
    let [Op::If(owner)] = root else {
        return Err(String::from("one actual If"));
    };
    let [branch] = owner.branches.as_mut_slice() else {
        return Err(String::from("one actual branch"));
    };
    Ok(take_carrier_key(branch))
}

fn bindings<'w, 'a>(root: &'w [Op<'a>]) -> Check<&'w [BindingOp<'a>]> {
    let [Op::If(owner)] = root else {
        return Err(String::from("one actual If"));
    };
    let [branch] = owner.branches.as_slice() else {
        return Err(String::from("one actual branch"));
    };
    let [op] = branch.region.ops.as_slice() else {
        return Err(String::from("one actual carrier"));
    };
    match op {
        Op::Element(owner) => Ok(&owner.bindings),
        Op::Component(owner) => Ok(&owner.bindings),
        Op::Slot(owner) => Ok(&owner.bindings),
        _ => Err(String::from("actual carrier surface")),
    }
}

fn without_static_key(mut page: Page) -> Check<Page> {
    let [DumpOp::If(owner)] = page.ops.as_mut_slice() else {
        return Err(String::from("whole If page"));
    };
    let [branch] = owner.branches.as_mut_slice() else {
        return Err(String::from("whole branch page"));
    };
    let [op] = branch.ops.as_mut_slice() else {
        return Err(String::from("whole carrier page"));
    };
    match op {
        DumpOp::Element(owner) => owner.attributes.retain(|attribute| attribute.name != "key"),
        DumpOp::Component(owner) => owner.attributes.retain(|attribute| attribute.name != "key"),
        DumpOp::Slot(owner) => owner.attributes.retain(|attribute| attribute.name != "key"),
        _ => return Err(String::from("whole carrier page kind")),
    }
    Ok(page)
}

fn attribute<'a>(
    source: &'a str,
    name: &'a str,
    value: Option<&'a str>,
    token: &str,
) -> Check<Attribute<'a>> {
    Ok(Attribute {
        name,
        value,
        span: span(source, token)?,
    })
}

fn bind<'a>(
    arena: &'a Allocator,
    source: &'a str,
    name: DynamicName<'a>,
    value: &'a str,
    token: &str,
) -> Check<BindingOp<'a>> {
    Ok(BindingOp::Bind(Box::new_in(
        BindOp {
            name: Some(name),
            modifiers: Vec::new_in(&arena),
            value: Some(js(arena, source, value)?),
            span: span(source, token)?,
        },
        &arena,
    )))
}

#[test]
fn static_extraction_preserves_the_whole_neighboring_branch() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<p title='before' key='chosen' id='after'/>";
    let extent = span(source, source)?;
    let attrs = Vec::from_iter_in(
        [
            attribute(source, "title", Some("before"), "title='before'")?,
            attribute(source, "key", Some("chosen"), "key='chosen'")?,
            attribute(source, "id", Some("after"), "id='after'")?,
        ],
        &arena,
    );
    let op = carrier(arena, Carrier::Element, attrs, Vec::new_in(&arena), extent);
    let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
    let before = Page::of(&tree);
    assert_eq!(
        take(&mut tree)?,
        Some(BranchKey {
            kind: BranchKeyKind::Static(Some("chosen".into())),
            span: span(source, "key='chosen'")?,
        })
    );
    assert_eq!(Page::of(&tree), without_static_key(before)?);
    assert_eq!(take(&mut tree)?, None);
    Ok(())
}

#[test]
fn earliest_dynamic_keeps_its_original_index_and_retained_ast() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<p :title='label' :key='id' key='late'/>";
    let extent = span(source, source)?;
    let ops = Vec::from_iter_in(
        [
            bind(
                arena,
                source,
                DynamicName::Static("title"),
                "label",
                ":title='label'",
            )?,
            bind(arena, source, DynamicName::Static("key"), "id", ":key='id'")?,
        ],
        &arena,
    );
    let op = carrier(
        arena,
        Carrier::Element,
        Vec::from_iter_in(
            [attribute(source, "key", Some("late"), "key='late'")?],
            &arena,
        ),
        ops,
        extent,
    );
    let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
    let before = Page::of(&tree);
    let [BindingOp::Bind(_), BindingOp::Bind(key)] = bindings(&tree)? else {
        return Err(String::from("two real bindings"));
    };
    let Some(ExprRef::Js(original)) = key.value else {
        return Err(String::from("real key AST"));
    };
    let original = original as *const JsExpr<'_>;
    let key_pointer = &**key as *const BindOp<'_>;
    assert_eq!(
        take(&mut tree)?,
        Some(BranchKey {
            kind: BranchKeyKind::Dynamic {
                source: "id".into(),
                bind_index: Some(1)
            },
            span: span(source, ":key='id'")?,
        })
    );
    assert_eq!(Page::of(&tree), before);
    let [BindingOp::Bind(_), BindingOp::Bind(key)] = bindings(&tree)? else {
        return Err(String::from("bindings retained"));
    };
    let Some(ExprRef::Js(after)) = key.value else {
        return Err(String::from("retained AST"));
    };
    assert!(core::ptr::eq(original, after));
    assert!(core::ptr::eq(key_pointer, &**key));
    Ok(())
}

#[test]
fn earliest_static_removes_only_its_attribute_and_keeps_dynamic_binding() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<p key='first' :key='id'/>";
    let extent = span(source, source)?;
    let op = carrier(
        arena,
        Carrier::Element,
        Vec::from_iter_in(
            [attribute(source, "key", Some("first"), "key='first'")?],
            &arena,
        ),
        Vec::from_iter_in(
            [bind(
                arena,
                source,
                DynamicName::Static("key"),
                "id",
                ":key='id'",
            )?],
            &arena,
        ),
        extent,
    );
    let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
    let before = Page::of(&tree);
    assert_eq!(
        take(&mut tree)?,
        Some(BranchKey {
            kind: BranchKeyKind::Static(Some("first".into())),
            span: span(source, "key='first'")?,
        })
    );
    assert_eq!(Page::of(&tree), without_static_key(before)?);
    Ok(())
}

#[test]
fn mismatched_carrier_span_and_multiple_roots_never_extract_or_mutate() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<template><p key='one'/><p key='two'/></template>";
    let first = span(source, "<p key='one'/>")?;
    let second = span(source, "<p key='two'/>")?;
    let first_op = || -> Check<Op<'_>> {
        Ok(carrier(
            arena,
            Carrier::Element,
            Vec::from_iter_in(
                [attribute(source, "key", Some("one"), "key='one'")?],
                &arena,
            ),
            Vec::new_in(&arena),
            first,
        ))
    };
    let mut foreign = root(
        arena,
        Vec::from_iter_in([first_op()?], &arena),
        span(source, source)?,
    );
    let before = Page::of(&foreign);
    assert_eq!(take(&mut foreign)?, None);
    assert_eq!(Page::of(&foreign), before);
    let second_op = carrier(
        arena,
        Carrier::Element,
        Vec::from_iter_in(
            [attribute(source, "key", Some("two"), "key='two'")?],
            &arena,
        ),
        Vec::new_in(&arena),
        second,
    );
    let mut multiple = root(
        arena,
        Vec::from_iter_in([first_op()?, second_op], &arena),
        Span::new(first.start, second.end),
    );
    let before = Page::of(&multiple);
    assert_eq!(take(&mut multiple)?, None);
    assert_eq!(Page::of(&multiple), before);
    Ok(())
}

#[test]
fn element_component_and_slot_share_static_and_dynamic_receivers() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<p key='chosen' :key='id'/>";
    let extent = span(source, source)?;
    for kind in [Carrier::Element, Carrier::Component, Carrier::Slot] {
        let op = carrier(
            arena,
            kind,
            Vec::from_iter_in(
                [attribute(source, "key", Some("chosen"), "key='chosen'")?],
                &arena,
            ),
            Vec::from_iter_in(
                [bind(
                    arena,
                    source,
                    DynamicName::Static("key"),
                    "id",
                    ":key='id'",
                )?],
                &arena,
            ),
            extent,
        );
        let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
        let before = Page::of(&tree);
        assert_eq!(
            take(&mut tree)?,
            Some(BranchKey {
                kind: BranchKeyKind::Static(Some("chosen".into())),
                span: span(source, "key='chosen'")?,
            })
        );
        assert_eq!(Page::of(&tree), without_static_key(before)?);
        let before = Page::of(&tree);
        assert_eq!(
            take(&mut tree)?,
            Some(BranchKey {
                kind: BranchKeyKind::Dynamic {
                    source: "id".into(),
                    bind_index: Some(0)
                },
                span: span(source, ":key='id'")?,
            })
        );
        assert_eq!(Page::of(&tree), before);
    }
    Ok(())
}

#[test]
fn bare_static_and_legacy_dynamic_argument_preserve_key_kinds() -> Check<()> {
    let arena = Allocator::new();
    let arena = &arena;
    let source = "<p key/>";
    let extent = span(source, source)?;
    let op = carrier(
        arena,
        Carrier::Element,
        Vec::from_iter_in([attribute(source, "key", None, "key")?], &arena),
        Vec::new_in(&arena),
        extent,
    );
    let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
    let Some(key) = take(&mut tree)? else {
        return Err(String::from("bare key extracts"));
    };
    assert_eq!(
        key,
        BranchKey {
            kind: BranchKeyKind::Static(None),
            span: span(source, "key")?
        }
    );
    assert_eq!(key.collision_text(), None);
    let source = "<p :[key]='id'/>";
    let extent = span(source, source)?;
    let op = carrier(
        arena,
        Carrier::Element,
        Vec::new_in(&arena),
        Vec::from_iter_in(
            [bind(
                arena,
                source,
                DynamicName::Dynamic(js(arena, source, "key")?),
                "id",
                ":[key]='id'",
            )?],
            &arena,
        ),
        extent,
    );
    let mut tree = root(arena, Vec::from_iter_in([op], &arena), extent);
    let before = Page::of(&tree);
    assert_eq!(
        take(&mut tree)?,
        Some(BranchKey {
            kind: BranchKeyKind::Dynamic {
                source: "id".into(),
                bind_index: Some(0)
            },
            span: span(source, ":[key]='id'")?,
        })
    );
    assert_eq!(Page::of(&tree), before);
    Ok(())
}
