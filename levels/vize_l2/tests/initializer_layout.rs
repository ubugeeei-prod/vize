//! Original field types/order, independent of the new enum's unit variants.
use vize_l0::{Span, String};
use vize_l2::{
    file::{Declaration, DeclarationKind, InitializerKind, Namespace, ScopeId, ScriptUnitId},
    resolution::BindingId,
};

#[allow(dead_code)]
enum OriginalInitializerKind {
    PrimitiveLiteral,
    Function,
    Unknown,
}

#[allow(dead_code)]
struct OriginalDeclarationLayout {
    id: BindingId,
    unit: ScriptUnitId,
    scope: ScopeId,
    name: String,
    span: Span,
    namespace: Namespace,
    kind: DeclarationKind,
    initializer: OriginalInitializerKind,
    import_source: Option<String>,
    imported_name: Option<String>,
    direct_program: bool,
}

#[test]
fn refined_unit_variants_keep_the_original_enum_and_declaration_size_alignment() {
    assert_eq!(
        core::mem::size_of::<InitializerKind>(),
        core::mem::size_of::<OriginalInitializerKind>()
    );
    assert_eq!(
        core::mem::align_of::<InitializerKind>(),
        core::mem::align_of::<OriginalInitializerKind>()
    );
    assert_eq!(
        core::mem::size_of::<Declaration>(),
        core::mem::size_of::<OriginalDeclarationLayout>()
    );
    assert_eq!(
        core::mem::align_of::<Declaration>(),
        core::mem::align_of::<OriginalDeclarationLayout>()
    );
}
