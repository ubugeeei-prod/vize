//! Executable backend-observation traces for Impeto programs.
//!
//! The Lean reference is the normative semantics. This Rust mirror exists so
//! generated S3 fixtures can be checked by ordinary cargo tests before the
//! dedicated Lean lane compares the same text.

use core::fmt::{self, Write};

use vize_s0::String;

use crate::op::{EffectId, Op, OpKind, Program};

/// Backend interpretation to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceBackend {
    /// VDOM-shaped observations.
    Vdom,
    /// Vapor-shaped observations.
    Vapor,
}

impl TraceBackend {
    /// Stable trace spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vdom => "vdom",
            Self::Vapor => "vapor",
        }
    }
}

fn label(backend: TraceBackend, kind: OpKind) -> &'static str {
    match backend {
        TraceBackend::Vdom => match kind {
            OpKind::SetProp => "patch-prop",
            OpKind::SetDynamicProps => "patch-dynamic-props",
            OpKind::SetText => "set-text",
            OpKind::SetEvent => "patch-event",
            OpKind::SetHtml => "set-html",
            OpKind::SetTemplateRef => "set-template-ref",
            OpKind::InsertNode => "create-element",
            OpKind::PrependNode => "prepend-node",
            OpKind::Directive => "apply-directive",
            OpKind::If => "branch",
            OpKind::For => "iterate",
            OpKind::CreateComponent => "create-component",
            OpKind::SlotOutlet => "render-slot",
            OpKind::GetTextChild => "get-text-child",
            OpKind::ChildRef => "child-ref",
            OpKind::NextRef => "next-ref",
        },
        TraceBackend::Vapor => match kind {
            OpKind::SetProp => "assign-prop",
            OpKind::SetDynamicProps => "assign-dynamic-props",
            OpKind::SetText => "text-effect",
            OpKind::SetEvent => "listen",
            OpKind::SetHtml => "html-effect",
            OpKind::SetTemplateRef => "template-ref",
            OpKind::InsertNode => "create-node",
            OpKind::PrependNode => "prepend-node",
            OpKind::Directive => "directive-effect",
            OpKind::If => "conditional-effect",
            OpKind::For => "list-effect",
            OpKind::CreateComponent => "component-effect",
            OpKind::SlotOutlet => "slot-effect",
            OpKind::GetTextChild => "get-text-child",
            OpKind::ChildRef => "child-ref",
            OpKind::NextRef => "next-ref",
        },
    }
}

fn write_effect<W: Write>(writer: &mut W, effect: Option<EffectId>) -> fmt::Result {
    if let Some(effect) = effect {
        write!(writer, " effect#{}", effect.index())?;
    }
    Ok(())
}

fn write_event<W: Write>(writer: &mut W, backend: TraceBackend, op: &Op) -> fmt::Result {
    write!(
        writer,
        "{} op#{} {}",
        backend.as_str(),
        op.id.index(),
        label(backend, op.kind)
    )?;
    write_effect(writer, op.effect)?;
    writer.write_char('\n')
}

/// Write one backend interpretation for `program`.
pub fn write_backend_trace<W: Write>(
    writer: &mut W,
    backend: TraceBackend,
    program: &Program<'_>,
) -> fmt::Result {
    for op in &program.ops {
        write_event(writer, backend, op)?;
    }
    Ok(())
}

/// Write the combined VDOM-then-Vapor reference trace for `program`.
pub fn write_reference_trace<W: Write>(writer: &mut W, program: &Program<'_>) -> fmt::Result {
    write_backend_trace(writer, TraceBackend::Vdom, program)?;
    write_backend_trace(writer, TraceBackend::Vapor, program)
}

/// Render one backend trace as text.
#[must_use]
pub fn backend_trace_text(backend: TraceBackend, program: &Program<'_>) -> String {
    let mut text = String::default();
    write_backend_trace(&mut text, backend, program).expect("writing to String cannot fail");
    text
}

/// Render the combined reference trace as text.
#[must_use]
pub fn reference_trace_text(program: &Program<'_>) -> String {
    let mut text = String::default();
    write_reference_trace(&mut text, program).expect("writing to String cannot fail");
    text
}
