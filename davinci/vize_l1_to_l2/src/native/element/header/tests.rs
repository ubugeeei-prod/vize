use super::{Context, HeaderAdmission, NativeHoleKind, StructuralHeadMask};
use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::SurfaceChild;
use vize_l1::embed::Lang;
use vize_l2::artifact::Builder;
use vize_l2::op::{Namespace, Op};

fn context<'a>(a: &'a Allocator, source: &'a str) -> Context<'a> {
    Context {
        allocator: a,
        block: SourceRoot::new(source).unwrap().whole_block(),
        lang: Lang::Js,
        holes: Vec::new(),
        diagnostics: Vec::new(),
        embeds: Vec::new(),
        rejected_syntax: Vec::new(),
    }
}

#[test]
fn moved_prepared_header_keeps_original_ordinals_and_once_decoded_attribute_pointer() {
    let a = Allocator::default();
    let file = "<div title=\"a&amp;b\" v-if=\"ok\" plain=\"x\" v-for=\"item in items\" :id=\"item.id\">text</div>";
    let component = vize_l1::markup::parse_component(&a, file).unwrap();
    let [SurfaceChild::Element(carrier)] = component.tree.children.as_slice() else {
        panic!("actual native carrier");
    };
    let mut cx = context(&a, file);
    let mut builder = Builder::new(&a, file).unwrap();
    let decoded;
    {
        let mut region = builder.region();
        let header = cx.prepare_element_header(&mut region, carrier, Namespace::Html);
        assert!(core::ptr::eq(header.carrier, carrier.as_ref()));
        assert_eq!(header.admission, HeaderAdmission::Ready);
        assert_eq!(
            header
                .directives
                .iter()
                .map(|head| head.ordinal)
                .collect::<Vec<_>>(),
            [1, 3, 4]
        );
        decoded = header.attributes.first().unwrap().value.unwrap();
        assert_eq!(decoded, "a&b");
        let mask = cx.structural_mask(&header, &[1]).unwrap();
        cx.element_body(&mut region, header, mask, (Namespace::Html, None));
    }
    let artifact = builder.finish().unwrap();
    let [Op::Element(owner)] = artifact.root().ops.as_slice() else {
        panic!("real minted owner");
    };
    assert!(core::ptr::eq(
        owner.attributes.first().unwrap().value.unwrap().as_ptr(),
        decoded.as_ptr()
    ));
    assert_eq!(owner.attributes.len(), 2);
    assert_eq!(owner.bindings.len(), 1);
    assert_eq!(artifact.node_count(), 3);
    assert_eq!(cx.embeds.len(), 1);
    assert_eq!(cx.embeds.first().unwrap().syntax.source().text(), "item.id");
    // Masking v-if must preserve the actual same-carrier For directive.
    assert_eq!(cx.holes.len(), 1);
    assert_eq!(cx.holes.first().unwrap().kind, NativeHoleKind::Directive);
    let span = cx.holes.first().unwrap().span;
    assert_eq!(
        file.get(span.start as usize..span.end as usize),
        Some("v-for=\"item in items\"")
    );
}

#[test]
fn a_mask_for_another_actual_carrier_cannot_suppress_this_headers_directive() {
    let a = Allocator::default();
    let file = "<p v-if=\"a\">one</p><p v-if=\"b\">two</p>";
    let component = vize_l1::markup::parse_component(&a, file).unwrap();
    let [SurfaceChild::Element(first), SurfaceChild::Element(second)] =
        component.tree.children.as_slice()
    else {
        panic!("actual sibling carriers");
    };
    let mut cx = context(&a, file);
    let mut builder = Builder::new(&a, file).unwrap();
    {
        let mut region = builder.region();
        let first = cx.prepare_element_header(&mut region, first, Namespace::Html);
        let second = cx.prepare_element_header(&mut region, second, Namespace::Html);
        let mask = cx.structural_mask(&first, &[0]).unwrap();
        cx.element_body(&mut region, second, mask, (Namespace::Html, None));
    }
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 2);
    assert_eq!(cx.holes.len(), 2);
    assert_eq!(
        cx.holes.first().unwrap().kind,
        NativeHoleKind::DirectiveSyntax
    );
    assert_eq!(cx.holes.get(1).unwrap().kind, NativeHoleKind::Directive);
    assert!(cx.embeds.is_empty());
    let [Op::Element(owner)] = artifact.root().ops.as_slice() else {
        panic!("second carrier")
    };
    let [Op::Text(text)] = owner.children.ops.as_slice() else {
        panic!("retained second body")
    };
    assert_eq!(text.content, "two");
}

#[test]
fn invalid_source_ordinals_and_nonstructural_heads_refuse_before_any_node_or_embed_mint() {
    for attribute in [
        "title=\"x\"",
        ":id=\"bad +\"",
        "@click=\"bad +\"",
        "v-if:arg=\"bad +\"",
        "v-if.mod=\"bad +\"",
    ] {
        let a = Allocator::default();
        let file = alloc::format!("<p {attribute} v-if=\"good\">body</p>");
        let component = vize_l1::markup::parse_component(&a, &file).unwrap();
        let [SurfaceChild::Element(carrier)] = component.tree.children.as_slice() else {
            panic!("native carrier")
        };
        let mut cx = context(&a, &file);
        let mut builder = Builder::new(&a, &file).unwrap();
        {
            let mut region = builder.region();
            let header = cx.prepare_element_header(&mut region, carrier, Namespace::Html);
            for ordinals in [&[0][..], &[1, 1], &[20]] {
                assert!(
                    matches!(
                        cx.structural_mask(&header, ordinals),
                        Err(NativeHoleKind::DirectiveSyntax)
                    ),
                    "{attribute}: {ordinals:?}"
                );
            }
            assert!(cx.structural_mask(&header, &[1]).is_ok());
        }
        assert_eq!(builder.finish().unwrap().node_count(), 0);
        assert!(cx.embeds.is_empty());
    }
}

#[test]
fn prepared_pre_carrier_and_missing_owner_keep_original_facts_without_duplicate_admission() {
    for file in [
        "<template v-pre><p :id=\"bad +\">{{bad +}}</p>",
        "<div><p>kept</p>",
    ] {
        let a = Allocator::default();
        let component = vize_l1::markup::parse_component(&a, file).unwrap();
        let [SurfaceChild::Element(carrier)] = component.tree.children.as_slice() else {
            panic!("original carrier")
        };
        let mut cx = context(&a, file);
        let mut builder = Builder::new(&a, file).unwrap();
        {
            let mut region = builder.region();
            let header = cx.prepare_element_header(&mut region, carrier, Namespace::Html);
            let holes = cx.holes.len();
            let pre = header.admission == HeaderAdmission::PreCarrier;
            cx.element_body(
                &mut region,
                header,
                StructuralHeadMask::default(),
                (Namespace::Html, None),
            );
            assert_eq!(cx.holes.len(), holes);
            assert!(cx.embeds.is_empty());
            assert_eq!(pre, file.starts_with("<template"));
        }
        assert_eq!(
            builder.finish().unwrap().node_count(),
            if file.starts_with("<template") { 0 } else { 2 }
        );
    }
}
