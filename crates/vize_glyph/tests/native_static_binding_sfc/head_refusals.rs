use super::{
    failure_support::{document, prefix},
    options,
};
use vize_glyph::native_doc::{
    LineEnding, NativeTemplateRefusal, TemplateRefusal, UnsupportedSyntax, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};

#[test]
fn shorthand_dynamic_tail_preserves_earlier_original_name_doc_geometry_refusal_and_prefix_custody()
{
    let source = "<template><i :title='first'>{{1n}}</i><p v-if='ok' :[key]tail='a+b'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::Template(
            TemplateRefusal::SourceMismatch { offset: 41 },
        )),
        1,
        Span::new(68, 77),
    );
    assert!(owner.binding_failure().is_none());
}

#[test]
fn full_dynamic_tail_preserves_earlier_original_name_doc_geometry_refusal_and_prefix_custody() {
    let source = "<template><i :title='first'>{{1n}}</i><p v-if='ok' v-bind:[key]tail='a+b'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::Template(
            TemplateRefusal::SourceMismatch { offset: 41 },
        )),
        1,
        Span::new(74, 83),
    );
    assert!(owner.binding_failure().is_none());
}

#[test]
fn empty_static_argument_preserves_earlier_original_unsupported_name_doc_and_prefix_custody() {
    let source =
        "<template><i :title='first'>{{1n}}</i><p v-if='ok' :='a+b'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::Template(
            TemplateRefusal::Unsupported {
                offset: 41,
                syntax: UnsupportedSyntax::Directive,
            },
        )),
        1,
        Span::new(59, 68),
    );
    assert!(owner.binding_failure().is_none());
}
