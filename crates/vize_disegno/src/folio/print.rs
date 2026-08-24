//! Canonical printer for [`DisegnoFolio`].
//!
//! `Full` mode is the injective, parseable form; `Display` elides every
//! ` @start:end` span - line tails and the spans inside expression
//! payloads alike (a semantic elision, hand-written per the P2-4
//! boundary) - and changes nothing else. Indentation is two spaces per
//! nesting level; the fixed grouping under an element - attributes, then
//! bindings, then children - is part of the canonical form.

use core::fmt::{Result, Write};

use vize_s0::Span;

use super::DisegnoFolio;
use super::owned::{FolioAttribute, FolioBinding, FolioExpr, FolioName, FolioOp};
use crate::op::Namespace;
use vize_davinci::folio::FolioMode;

mod binding;

use binding::{print_attribute, print_binding};

pub(super) fn print<W: Write>(folio: &DisegnoFolio, w: &mut W, mode: FolioMode) -> Result {
    writeln!(w, "[disegno]")?;
    writeln!(w, "ops={}", folio.op_count())?;
    writeln!(w)?;

    if folio.ops.is_empty() {
        return Ok(());
    }
    writeln!(w, "[disegno.ops]")?;
    for op in &folio.ops {
        print_op(w, op, 0, mode)?;
    }
    writeln!(w)
}

pub(super) fn indent<W: Write>(w: &mut W, depth: usize) -> Result {
    for _ in 0..depth {
        w.write_str("  ")?;
    }
    Ok(())
}

/// Write the span tail in `Full` mode, nothing in `Display`, then the
/// newline either way.
pub(super) fn end_line<W: Write>(w: &mut W, span: Span, mode: FolioMode) -> Result {
    if mode == FolioMode::Full {
        write!(w, " @{}:{}", span.start, span.end)?;
    }
    w.write_char('\n')
}

/// Write one quoted string with the format's escapes.
pub(super) fn quoted<W: Write>(w: &mut W, text: &str) -> Result {
    w.write_char('"')?;
    for c in text.chars() {
        match c {
            '"' => w.write_str("\\\"")?,
            '\\' => w.write_str("\\\\")?,
            '\n' => w.write_str("\\n")?,
            '\r' => w.write_str("\\r")?,
            '\t' => w.write_str("\\t")?,
            other => w.write_char(other)?,
        }
    }
    w.write_char('"')
}

/// Write one expression payload: `js("…" @s:e)` / `opaque(reason "…" @s:e)`
/// / `foreign(dialect "…" @s:e)`; `Display` elides the inner span tail
/// exactly as it elides line tails.
pub(super) fn print_expr<W: Write>(w: &mut W, expr: &FolioExpr, mode: FolioMode) -> Result {
    let (head, source, span) = match expr {
        FolioExpr::Js { source, span } => {
            w.write_str("js(")?;
            ("", source, span)
        }
        FolioExpr::Opaque {
            reason,
            source,
            span,
        } => {
            w.write_str("opaque(")?;
            (reason.mnemonic(), source, span)
        }
        FolioExpr::Foreign {
            dialect,
            source,
            span,
        } => {
            w.write_str("foreign(")?;
            (dialect.as_str(), source, span)
        }
        FolioExpr::Filter { source, span } => {
            w.write_str("vue.filter(")?;
            ("", source, span)
        }
    };
    if !head.is_empty() {
        w.write_str(head)?;
        w.write_char(' ')?;
    }
    quoted(w, source.as_str())?;
    if mode == FolioMode::Full {
        write!(w, " @{}:{}", span.start, span.end)?;
    }
    w.write_char(')')
}

pub(super) fn print_name<W: Write>(w: &mut W, name: &FolioName, mode: FolioMode) -> Result {
    match name {
        FolioName::Static(text) => quoted(w, text.as_str()),
        FolioName::Dynamic(expr) => print_expr(w, expr, mode),
    }
}

fn print_op<W: Write>(w: &mut W, op: &FolioOp, depth: usize, mode: FolioMode) -> Result {
    match op {
        FolioOp::Element(element) => {
            indent(w, depth)?;
            write!(w, "ui.element {}", element.tag)?;
            match element.namespace {
                Namespace::Html => {}
                Namespace::Svg => w.write_str(" ns=svg")?,
                Namespace::MathMl => w.write_str(" ns=mathml")?,
            }
            end_line(w, element.span, mode)?;
            print_owner_body(
                w,
                &element.attributes,
                &element.bindings,
                &element.children,
                depth + 1,
                mode,
            )
        }
        FolioOp::Component(component) => {
            indent(w, depth)?;
            write!(w, "ui.component {}", component.name)?;
            end_line(w, component.span, mode)?;
            print_owner_body(
                w,
                &component.attributes,
                &component.bindings,
                &component.children,
                depth + 1,
                mode,
            )
        }
        FolioOp::Text(text) => {
            indent(w, depth)?;
            w.write_str("ui.text ")?;
            quoted(w, text.content.as_str())?;
            end_line(w, text.span, mode)
        }
        FolioOp::Interpolation(interpolation) => {
            indent(w, depth)?;
            w.write_str("ui.interpolation ")?;
            print_expr(w, &interpolation.expression, mode)?;
            end_line(w, interpolation.span, mode)
        }
        FolioOp::If(if_op) => {
            indent(w, depth)?;
            w.write_str("ui.if")?;
            end_line(w, if_op.span, mode)?;
            for branch in &if_op.branches {
                indent(w, depth + 1)?;
                w.write_str("branch")?;
                if let Some(condition) = &branch.condition {
                    w.write_char(' ')?;
                    print_expr(w, condition, mode)?;
                }
                end_line(w, branch.span, mode)?;
                for child in &branch.ops {
                    print_op(w, child, depth + 2, mode)?;
                }
            }
            Ok(())
        }
        FolioOp::For(for_op) => {
            indent(w, depth)?;
            w.write_str("ui.for source=")?;
            print_expr(w, &for_op.binding.source, mode)?;
            w.write_str(" value=")?;
            print_expr(w, &for_op.binding.value, mode)?;
            if let Some(key) = &for_op.binding.key {
                w.write_str(" key=")?;
                print_expr(w, key, mode)?;
            }
            if let Some(index) = &for_op.binding.index {
                w.write_str(" index=")?;
                print_expr(w, index, mode)?;
            }
            end_line(w, for_op.span, mode)?;
            for child in &for_op.ops {
                print_op(w, child, depth + 1, mode)?;
            }
            Ok(())
        }
        FolioOp::Slot(slot) => {
            indent(w, depth)?;
            w.write_str("ui.slot name=")?;
            print_name(w, &slot.name, mode)?;
            end_line(w, slot.span, mode)?;
            print_owner_body(
                w,
                &slot.attributes,
                &slot.bindings,
                &slot.fallback,
                depth + 1,
                mode,
            )
        }
    }
}

fn print_owner_body<W: Write>(
    w: &mut W,
    attributes: &[FolioAttribute],
    bindings: &[FolioBinding],
    children: &[FolioOp],
    depth: usize,
    mode: FolioMode,
) -> Result {
    for attribute in attributes {
        print_attribute(w, attribute, depth, mode)?;
    }
    for binding in bindings {
        print_binding(w, binding, depth, mode)?;
    }
    for child in children {
        print_op(w, child, depth, mode)?;
    }
    Ok(())
}
