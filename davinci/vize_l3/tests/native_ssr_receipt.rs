use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l3::decision::ssr::{
    NativeSsrBuildError, SsrPart, SsrUnsupported, build_native_ssr_file_decisions,
};

#[expect(
    clippy::unwrap_used,
    reason = "invalid original fixture must fail admission"
)]
fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let component = {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap()
    };
    let mut owner = NativeTemplateOwner::new(component).unwrap_or_else(|_| panic!("owner"));
    {
        let mut walk = owner.begin().unwrap();
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    core::hint::black_box(owner.finish())
}

#[test]
fn original_completed_owner_supplies_all_ssr_decisions_after_descriptor_drop() {
    for source in [
        "<template></template>",
        "<template>hé世界<!--original-->tail</template>",
        "<template><div><br/><p>hé</p></div></template>",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let file = output.file().unwrap();
        let original = file.artifact().provenance().to_vec();
        let receipt = build_native_ssr_file_decisions(output.view().unwrap()).unwrap();
        assert!(core::ptr::eq(receipt.owner(), &output));
        assert!(core::ptr::eq(receipt.file(), file));
        assert!(core::ptr::eq(receipt.artifact(), file.artifact()));
        assert!(core::ptr::eq(receipt.artifact().source(), source));
        assert_eq!(
            receipt.tables().nodes.len(),
            file.artifact().node_count() as usize
        );
        assert_eq!(receipt.artifact().provenance(), original);
        let facts = receipt.ssr().unwrap();
        assert!(facts.unsupported().is_empty());
        for part in facts.parts() {
            let span = match part {
                SsrPart::Open { element, .. } | SsrPart::Close { element, .. } => element.span,
                SsrPart::Text { text, .. } => text.span,
                SsrPart::Comment { comment, .. } => comment.span,
                SsrPart::Interpolation { interpolation, .. } => interpolation.span,
            };
            assert!(source.get(span.start as usize..span.end as usize).is_some());
        }
    }
}

#[test]
fn every_authentic_style_block_refuses_before_output_authority_is_returned() {
    for source in [
        "<template><div/></template><style></style>",
        "<style scoped>div{color:red}</style><template><div/></template>",
        "<template><div/></template><style lang=scss>$x:red;</style>",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        assert!(output.selected().has_styles());
        let Err(error) = build_native_ssr_file_decisions(output.view().unwrap()) else {
            panic!("styled source cannot mint this bounded receipt");
        };
        assert_eq!(
            error,
            NativeSsrBuildError::StyledSource {
                span: Span::new(0, source.len() as u32),
            }
        );
    }
}

#[test]
fn original_search_role_remains_an_exact_whole_view_semantic_refusal() {
    let arena = Allocator::default();
    let source = "<template><div><search/></div></template>";
    let output = completed(&arena, source);
    let receipt = build_native_ssr_file_decisions(output.view().unwrap()).unwrap();
    let rejected = &receipt.ssr().unwrap().unsupported()[0];
    assert_eq!(rejected.reason, SsrUnsupported::ElementSemantics);
    assert_eq!(rejected.node.index(), 1);
    assert_eq!(
        source.get(rejected.span.start as usize..rejected.span.end as usize),
        Some("<search/>")
    );
}
