//! Link one sealed original setup body into the prepared component fragment.

use crate::write::{LinkSink, Writer};
use vize_l0::{SourceBlock, Span};
use vize_l2::file::{BindingRef, DeclarationKind};
use vize_l2::lang::js::{NativeSelectedSetup, SetupAnnotation, VueSetup};

pub const COMPONENT_BINDING: &str = "_sfc_main";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupEmitErrorKind {
    MissingBinding,
    GeneratedBindingCollision,
    InvalidTypeAnnotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupEmitError {
    pub span: Span,
    pub kind: SetupEmitErrorKind,
}

/// Copy original runtime segments once, omitting only sealed annotation spans.
/// Original syntax/source are retained and every copied segment keeps full-file links.
/// Only genuine root setup bindings are returned. Mutable lexical bindings use
/// getters/setters; immutable constants expose only a getter. No caller-supplied
/// setup state or initializer lowering exists.
/// Generated-name refusals happen before a writer or partial fragment is created.
pub fn emit_setup<L: LinkSink>(
    setup: &VueSetup<'_, '_, '_, '_>,
) -> Result<Writer<L>, SetupEmitError> {
    emit_body(setup)
}

/// Emit the normally owned original setup; no caller Program/File/source pairing.
pub fn emit_selected_setup<L: LinkSink>(
    setup: &NativeSelectedSetup<'_, '_>,
) -> Result<Writer<L>, SetupEmitError> {
    emit_body(setup)
}

// Only genuine sealed setup owners implement this private segment projection.
// It borrows existing declaration/annotation rows, without walking the AST.
trait SetupInput<'owner, 'arena> {
    fn source(&self) -> SourceBlock<'arena>;
    fn bindings(&self) -> impl Iterator<Item = BindingRef<'owner, 'arena>>;
    fn type_annotations(&self) -> impl Iterator<Item = SetupAnnotation<'owner, 'arena>>;
}
impl<'owner, 'arena> SetupInput<'owner, 'arena> for VueSetup<'owner, '_, '_, 'arena> {
    fn source(&self) -> SourceBlock<'arena> {
        self.source()
    }
    fn bindings(&self) -> impl Iterator<Item = BindingRef<'owner, 'arena>> {
        self.bindings()
    }
    fn type_annotations(&self) -> impl Iterator<Item = SetupAnnotation<'owner, 'arena>> {
        self.type_annotations()
    }
}
impl<'owner, 'arena> SetupInput<'owner, 'arena> for NativeSelectedSetup<'owner, 'arena> {
    fn source(&self) -> SourceBlock<'arena> {
        self.source()
    }
    fn bindings(&self) -> impl Iterator<Item = BindingRef<'owner, 'arena>> {
        self.bindings()
    }
    fn type_annotations(&self) -> impl Iterator<Item = SetupAnnotation<'owner, 'arena>> {
        self.type_annotations()
    }
}
fn emit_body<'owner, 'arena, L: LinkSink>(
    setup: &impl SetupInput<'owner, 'arena>,
) -> Result<Writer<L>, SetupEmitError> {
    let fail = |span, kind| SetupEmitError { span, kind };
    for binding in setup.bindings() {
        let declaration = binding
            .declaration()
            .ok_or_else(|| fail(setup.source().span(), SetupEmitErrorKind::MissingBinding))?;
        if matches!(declaration.name.as_str(), "Object" | "__value" | "__v_raw") {
            return Err(fail(
                declaration.span,
                SetupEmitErrorKind::GeneratedBindingCollision,
            ));
        }
    }
    let source = setup.source();
    let mut previous = source.start();
    for annotation in setup.type_annotations() {
        let span = annotation.span();
        if span.start < previous || span.start == span.end || !source.contains_block_span(span) {
            return Err(fail(span, SetupEmitErrorKind::InvalidTypeAnnotation));
        }
        previous = span.end;
    }
    let mut writer = Writer::default();
    writer.push("const _sfc_main = {\n  setup(__props, { expose: __expose }) {\n    __expose();\n");
    let mut cursor = source.start();
    for annotation in setup.type_annotations() {
        let span = annotation.span();
        if cursor != span.start {
            let text = source
                .source()
                .get((cursor - source.start()) as usize..(span.start - source.start()) as usize)
                .ok_or_else(|| fail(span, SetupEmitErrorKind::InvalidTypeAnnotation))?;
            writer.push_linked(text, Span::new(cursor, span.start));
        }
        cursor = span.end;
    }
    let tail = source
        .source()
        .get((cursor - source.start()) as usize..)
        .ok_or_else(|| fail(source.span(), SetupEmitErrorKind::InvalidTypeAnnotation))?;
    writer.push_linked(tail, Span::new(cursor, source.end()));
    // The copied source may end in a line comment or omit its last semicolon.
    // Generated code always starts a separate statement on its own line.
    writer.push("\n;\n    const __returned__ = {\n");
    for binding in setup.bindings() {
        let declaration = binding
            .declaration()
            .ok_or_else(|| fail(source.span(), SetupEmitErrorKind::MissingBinding))?;
        let name = declaration.name.as_str();
        let span = declaration.span;
        writer.push("      get ");
        writer.push_named(name, span, name);
        writer.push("() { return ");
        writer.push_named(name, span, name);
        writer.push(" },\n");
        if matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ) {
            writer.push("      set ");
            writer.push_named(name, span, name);
            writer.push("(__value) { ");
            writer.push_named(name, span, name);
            writer.push(" = __value },\n");
        }
    }
    writer.push("    };\n    Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true });\n    return __returned__;\n  }\n}\n");
    Ok(writer)
}
