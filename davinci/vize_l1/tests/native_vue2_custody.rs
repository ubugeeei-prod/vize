//! Actual original Vue 2 CST occurrences joined to retained filter-chain syntax.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "complete original source/token/AST custody laws fail on invalid evidence"
)]

use vize_l0::{Allocator, SourceRoot};
use vize_l1::SurfaceChild;
use vize_l1::dialect::LegacyVueVersion;
use vize_l1::dialect::vue2::surface::{self, TextChildren, TextRefusal, TextView};
use vize_l1::render::check_fidelity;

#[path = "native_vue2_custody/mod.rs"]
mod cases;

fn visits<'o, 'a>(
    owner: &'o surface::ComponentParse<'a>,
    children: TextChildren<'o, 'a>,
    observed: &mut Vec<Result<TextView<'o, 'a>, TextRefusal>>,
) {
    for child in children {
        if let Some(children) = child.children() {
            visits(owner, children, observed);
        } else if matches!(child.surface(), SurfaceChild::Interpolation(_)) {
            observed.push(owner.text_for(child));
        }
    }
}

#[test]
fn complete_nonzero_unicode_packet_keeps_original_occurrences_and_every_ast() {
    let root = "前<script>const 雪=1</script><template>甲{{ &#160;雪 | wrap('後', 1 + 2,) }}乙<main>{{ value | add(2,) }}{{ value | add(2,) }}</main>{{ tail | upper( ) }}</template>後";
    let start = root.find("甲").unwrap();
    let end = root.find("</template>").unwrap();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(&root[start..end], start as u32)
        .unwrap();
    let arena = Allocator::default();
    let owner = surface::parse_component_with_authored_block(&arena, block);
    assert_eq!(owner.version(), LegacyVueVersion::V2);
    assert_eq!(owner.block(), block);
    assert!(owner.errors().is_empty());
    assert!(owner.unsupported().is_empty());
    assert!(owner.authored().is_none());
    assert!(owner.authored_children().is_none());
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    let kinds: Vec<_> = owner
        .children()
        .map(|child| match child.surface() {
            SurfaceChild::Text(_) => "text",
            SurfaceChild::Interpolation(_) => "interpolation",
            SurfaceChild::Element(_) => "element",
            _ => "unexpected",
        })
        .collect();
    assert_eq!(
        kinds,
        ["text", "interpolation", "text", "element", "interpolation"]
    );
    let expected = [
        (
            " &#160;雪 | wrap('後', 1 + 2,) ",
            "雪",
            vec![("wrap", vec!["'後'", "1 + 2"])],
            None,
            1,
        ),
        (
            " value | add(2,) ",
            "value",
            vec![("add", vec!["2"])],
            Some("main"),
            0,
        ),
        (
            " value | add(2,) ",
            "value",
            vec![("add", vec!["2"])],
            Some("main"),
            1,
        ),
        (
            " tail | upper( ) ",
            "tail",
            vec![("upper", vec![])],
            None,
            4,
        ),
    ];
    let mut observed = Vec::new();
    visits(&owner, owner.children(), &mut observed);
    assert_eq!(observed.len(), expected.len());
    assert_eq!(owner.bindings().len(), expected.len());
    for (index, (observed, (raw, base, filters, parent, ordinal))) in
        observed.iter().zip(expected).enumerate()
    {
        let view = observed.as_ref().unwrap();
        assert!(core::ptr::eq(view.binding(), &owner.bindings()[index]));
        assert!(core::ptr::eq(
            view.chain(),
            owner.bindings()[index].admitted().unwrap()
        ));
        assert!(core::ptr::eq(view.child().component(), &owner));
        assert_eq!(view.child().ordinal(), ordinal);
        assert_eq!(view.child().parent_element().map(|node| node.tag()), parent);
        let original = match view.child().parent_element() {
            Some(parent) => &parent.children[ordinal],
            None => &owner.tree().children[ordinal],
        };
        assert!(core::ptr::eq(view.child().surface(), original));
        let SurfaceChild::Interpolation(node) = original else {
            panic!("original interpolation")
        };
        assert!(core::ptr::eq(
            view.binding().raw_content(),
            node.content.text
        ));
        assert_eq!(view.binding().raw_content(), raw);
        assert_eq!(view.chain().base().source().text(), base);
        let actual: Vec<_> = view
            .chain()
            .filters()
            .iter()
            .map(|filter| {
                (
                    filter.name().text(),
                    filter
                        .arguments()
                        .iter()
                        .map(|arg| arg.source().text())
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(actual, filters);
        let span = view.binding().span();
        assert_eq!(
            &root[span.start as usize..span.end as usize],
            format!("{{{{{raw}}}}}")
        );
        let original_chain = owner.bindings()[index].admitted().unwrap();
        let original_syntax = core::iter::once(original_chain.base()).chain(
            original_chain
                .filters()
                .iter()
                .flat_map(|filter| filter.arguments()),
        );
        for (syntax, original) in core::iter::once(view.chain().base())
            .chain(
                view.chain()
                    .filters()
                    .iter()
                    .flat_map(|filter| filter.arguments()),
            )
            .zip(original_syntax)
        {
            assert!(syntax.hole().is_none());
            assert_eq!(syntax.grammar().shape, vize_l1::embed::Shape::Expr);
            assert_eq!(syntax.grammar().lang, vize_l1::embed::Lang::Js);
            assert!(core::ptr::eq(syntax.source().authored_root(), root));
            assert!(block.contains_block_span(syntax.source().span()));
            assert!(core::ptr::eq(syntax, original));
            assert!(core::ptr::eq(
                syntax.expression().unwrap(),
                original.expression().unwrap()
            ));
        }
    }
    let first = owner.children().nth(3).unwrap();
    let mut children = first.children().unwrap();
    let left = owner.text_for(children.next().unwrap()).unwrap();
    let right = owner.text_for(children.next().unwrap()).unwrap();
    assert_eq!(left.binding().raw_content(), right.binding().raw_content());
    assert!(!core::ptr::eq(left.binding(), right.binding()));
    assert!(!core::ptr::eq(
        left.chain().base().expression().unwrap(),
        right.chain().base().expression().unwrap()
    ));
    assert_ne!(left.binding().span(), right.binding().span());
}

#[test]
fn moved_owners_keep_original_storage_and_short_reborrow_identity() {
    let arena = Allocator::default();
    let source = "{{ 雪 | wrap(&quot;後&quot;, 2,) }}";
    let original = surface::parse_component(&arena, source).unwrap();
    let child_pointer = &original.tree().children[0] as *const SurfaceChild<'_>;
    let ast_pointer = original.bindings()[0]
        .admitted()
        .unwrap()
        .base()
        .expression()
        .unwrap() as *const _;
    let argument_pointer = original.bindings()[0].admitted().unwrap().filters()[0].arguments()[0]
        .expression()
        .unwrap() as *const _;
    let moved = Box::new(original);
    let child = moved.children().next().unwrap();
    {
        let view = moved.text_for(child.reborrow()).unwrap();
        assert!(core::ptr::eq(view.child().surface(), child_pointer));
        assert!(core::ptr::eq(
            view.chain().base().expression().unwrap(),
            ast_pointer
        ));
        assert!(core::ptr::eq(
            view.chain().filters()[0].arguments()[0]
                .expression()
                .unwrap(),
            argument_pointer
        ));
        let argument = &view.chain().filters()[0].arguments()[0];
        assert_eq!(argument.source().text(), "\"後\"");
        assert!(argument.source().decode_map().is_some());
        let span = argument.source().span();
        assert_eq!(
            &source[span.start as usize..span.end as usize],
            "&quot;後&quot;"
        );
    }
    drop(child);
    let mut owners = Vec::new();
    owners.push(*moved);
    for _ in 0..64 {
        owners.push(surface::parse_component(&arena, "{{ other }}").unwrap());
    }
    let moved = &owners[0];
    let view = moved.text_for(moved.children().next().unwrap()).unwrap();
    assert!(core::ptr::eq(view.child().surface(), child_pointer));
    assert!(core::ptr::eq(
        view.chain().base().expression().unwrap(),
        ast_pointer
    ));
    assert!(core::ptr::eq(
        view.chain().filters()[0].arguments()[0]
            .expression()
            .unwrap(),
        argument_pointer
    ));
}

#[test]
fn every_utf8_prefix_retains_complete_original_source_and_refusal_observations() {
    let source = "<a>前{{ 雪 &#124; wrap(&quot;後&quot;, 2,) }}<a>{{ next | upper }}</a>後</a>";
    for end in (0..=source.len()).filter(|&end| source.is_char_boundary(end)) {
        let root = format!(
            "前<script>const 雪=1</script><template>{}</template>後",
            &source[..end]
        );
        let start = root.find("<template>").unwrap() + "<template>".len();
        let block_end = root.find("</template>").unwrap();
        let block = SourceRoot::new(&root)
            .unwrap()
            .block(&root[start..block_end], start as u32)
            .unwrap();
        let arena = Allocator::default();
        let owner = surface::parse_component_with_authored_block(&arena, block);
        assert_eq!(check_fidelity(owner.tree()), Ok(()), "{end}");
        if let Some(authored) = owner.authored() {
            assert_eq!(check_fidelity(authored), Ok(()), "{end}");
        }
        for binding in owner.bindings() {
            assert!(block.contains_block_span(binding.span()), "{end}");
            assert!(
                core::ptr::eq(binding.source().authored_root(), root.as_str()),
                "{end}"
            );
            assert!(block.span_of(binding.raw_content()).is_some(), "{end}");
            for boundary in binding.boundaries() {
                assert!(block.contains_block_span(boundary.span), "{end}");
            }
        }
        for pair in owner.bindings().windows(2) {
            assert!(pair[0].span().start < pair[1].span().start, "{end}");
        }
        let mut observed = Vec::new();
        visits(&owner, owner.children(), &mut observed);
        for observed in observed {
            if let Ok(view) = observed {
                assert!(
                    core::ptr::eq(view.chain(), view.binding().admitted().unwrap()),
                    "{end}"
                );
                let SurfaceChild::Interpolation(node) = view.child().surface() else {
                    panic!("{end}")
                };
                assert!(
                    core::ptr::eq(view.binding().raw_content(), node.content.text),
                    "{end}"
                );
                assert!(owner.errors().is_empty(), "{end}");
                assert!(owner.unsupported().is_empty(), "{end}");
            }
        }
    }
}
