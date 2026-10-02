use super::{Context, HeaderAdmission, NativeHoleKind, StructuralHeadMask};
use crate::native::construction::Diagnostic;
use crate::native::{NativeComponent, NativeObservations};
use alloc::vec::Vec;
use vize_l0::{Allocator, SourceRoot, cstr};
use vize_l1::SurfaceChild;
use vize_l1::embed::Lang;
use vize_l2::artifact::Builder;
use vize_l2::op::{Namespace, Op};

fn context<'component, 'a>(
    a: &'a Allocator,
    component: &'component NativeComponent<'a>,
    observations: &'component mut NativeObservations<'a>,
) -> Context<'component, 'a> {
    Context {
        allocator: a,
        component,
        lang: Lang::Js,
        holes: &mut observations.holes,
        diagnostics: &mut observations.diagnostics,
        embeds: &mut observations.embeds,
        rejected_syntax: &mut observations.rejected_syntax,
    }
}

#[test]
fn moved_prepared_header_keeps_original_ordinals_and_once_decoded_attribute_pointer() {
    let a = Allocator::default();
    let file = "<div title=\"a&amp;b\" v-if=\"ok\" plain=\"x\" v-for=\"item in items\" :id=\"item.id\">text</div>";
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let [SurfaceChild::Element(carrier)] = component.carrier().tree.children.as_slice() else {
        panic!("actual native carrier");
    };
    let mut observations = NativeObservations::new();
    let mut cx = context(&a, &component, &mut observations);
    let mut builder = Builder::new(&a, file).unwrap();
    let decoded;
    {
        let mut region = builder.region();
        let mut region = Diagnostic::new(&mut region);
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
    let component =
        NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
    let [SurfaceChild::Element(first), SurfaceChild::Element(second)] =
        component.carrier().tree.children.as_slice()
    else {
        panic!("actual sibling carriers");
    };
    let mut observations = NativeObservations::new();
    let mut cx = context(&a, &component, &mut observations);
    let mut builder = Builder::new(&a, file).unwrap();
    {
        let mut region = builder.region();
        let mut region = Diagnostic::new(&mut region);
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
    let [_, second] = cx.holes.as_slice() else {
        panic!("two retained carrier refusals");
    };
    assert_eq!(second.kind, NativeHoleKind::Directive);
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
        let file = cstr!("<p {attribute} v-if=\"good\">body</p>");
        let component =
            NativeComponent::parse_in(&a, SourceRoot::new(&file).unwrap().whole_block()).unwrap();
        let [SurfaceChild::Element(carrier)] = component.carrier().tree.children.as_slice() else {
            panic!("native carrier")
        };
        let mut observations = NativeObservations::new();
        let mut cx = context(&a, &component, &mut observations);
        let mut builder = Builder::new(&a, &file).unwrap();
        {
            let mut region = builder.region();
            let mut region = Diagnostic::new(&mut region);
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
        let component =
            NativeComponent::parse_in(&a, SourceRoot::new(file).unwrap().whole_block()).unwrap();
        let [SurfaceChild::Element(carrier)] = component.carrier().tree.children.as_slice() else {
            panic!("original carrier")
        };
        let mut observations = NativeObservations::new();
        let mut cx = context(&a, &component, &mut observations);
        let mut builder = Builder::new(&a, file).unwrap();
        {
            let mut region = builder.region();
            let mut region = Diagnostic::new(&mut region);
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
