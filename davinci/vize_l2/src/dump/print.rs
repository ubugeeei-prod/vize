//! Canonical printer for [`Page`].
//!
//! `Full` mode is the injective, parseable form; `Display` elides every
//! ` @start:end` span - line tails and the spans inside expression
//! payloads alike (a semantic elision, hand-written per the P2-4
//! boundary) - and changes nothing else. Indentation is two spaces per
//! nesting level; the fixed grouping under an element - attributes, then
//! bindings, then children - is part of the canonical form.

use core::fmt::{Result, Write};

use vize_l0::{Span, ensure_sufficient_stack};

use crate::dump::Page;
use crate::dump::codec::Grammar;
use crate::dump::owned::{Attribute, Binding, Expr, Name, Op};
use crate::op::Namespace;
use vize_davinci::dump::Mode as DumpMode;
use vize_davinci::key::rebase;

mod binding;

use binding::{print_attribute, print_binding};

/// How a page prints: its grammar and mode, plus the offset spans are rebased
/// to. The public [`Dump`](vize_davinci::dump::Dump) print always uses
/// base `0` (spans verbatim); only the P5-1a key feed passes a block start.
#[derive(Debug, Clone, Copy)]
pub(super) struct Style {
    mode: DumpMode,
    base: u32,
    grammar: Grammar,
}

impl Style {
    /// The current dump: spans verbatim.
    pub(super) const fn current(mode: DumpMode) -> Self {
        Self {
            mode,
            base: 0,
            grammar: Grammar::CurrentV2,
        }
    }

    /// The explicitly historical published wire: spans verbatim.
    pub(super) const fn historical_v1(mode: DumpMode) -> Self {
        Self {
            mode,
            base: 0,
            grammar: Grammar::HistoricalV1,
        }
    }

    /// The key feed: the `Full` form with spans rebased to `block_start`.
    pub(super) const fn keyed(block_start: u32) -> Self {
        Self {
            mode: DumpMode::Full,
            base: block_start,
            grammar: Grammar::CurrentV2,
        }
    }
}

pub(super) fn print<W: Write>(page: &Page, w: &mut W, mode: Style) -> Result {
    writeln!(w, "[{}]", mode.grammar.header())?;
    writeln!(w, "ops={}", page.op_count())?;
    writeln!(w)?;

    if page.ops.is_empty() {
        return Ok(());
    }
    writeln!(w, "[{}]", mode.grammar.ops_header())?;
    for op in &page.ops {
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
pub(super) fn end_line<W: Write>(w: &mut W, span: Span, mode: Style) -> Result {
    span_tail(w, span, mode)?;
    w.write_char('\n')
}

/// Write ` @start:end` in `Full` mode, nothing in `Display`. Spans are
/// rebased to the style's base; one reaching before the base (never in a
/// well-formed page, and impossible at base `0`) prints absolute as
/// ` @^start:end`, so the key feed stays injective instead of saturating.
fn span_tail<W: Write>(w: &mut W, span: Span, mode: Style) -> Result {
    if mode.mode != DumpMode::Full {
        return Ok(());
    }
    match rebase(span, mode.base) {
        Some(relative) => write!(w, " @{}:{}", relative.start, relative.end),
        None => write!(w, " @^{}:{}", span.start, span.end),
    }
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
pub(super) fn print_expr<W: Write>(w: &mut W, expr: &Expr, mode: Style) -> Result {
    let (head, source, span) = match expr {
        Expr::Js { source, span } => {
            w.write_str("js(")?;
            ("", source, span)
        }
        Expr::Opaque {
            reason,
            source,
            span,
        } => {
            w.write_str("opaque(")?;
            (reason.mnemonic(), source, span)
        }
        Expr::Foreign {
            dialect,
            source,
            span,
        } => {
            w.write_str("foreign(")?;
            (dialect.as_str(), source, span)
        }
        Expr::Filter { source, span } => {
            w.write_str("vue.filter(")?;
            ("", source, span)
        }
    };
    if !head.is_empty() {
        w.write_str(head)?;
        w.write_char(' ')?;
    }
    quoted(w, source.as_str())?;
    span_tail(w, *span, mode)?;
    w.write_char(')')
}

pub(super) fn print_name<W: Write>(w: &mut W, name: &Name, mode: Style) -> Result {
    match name {
        Name::Static(text) => quoted(w, text.as_str()),
        Name::Dynamic(expr) => print_expr(w, expr, mode),
    }
}

fn print_op<W: Write>(w: &mut W, op: &Op, depth: usize, mode: Style) -> Result {
    ensure_sufficient_stack(|| print_op_guarded(w, op, depth, mode))
}

fn print_op_guarded<W: Write>(w: &mut W, op: &Op, depth: usize, mode: Style) -> Result {
    match op {
        Op::Element(element) => {
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
        Op::Component(component) => {
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
        Op::Text(text) => {
            indent(w, depth)?;
            w.write_str("ui.text ")?;
            quoted(w, text.content.as_str())?;
            end_line(w, text.span, mode)
        }
        Op::Interpolation(interpolation) => {
            indent(w, depth)?;
            w.write_str("ui.interpolation ")?;
            print_expr(w, &interpolation.expression, mode)?;
            end_line(w, interpolation.span, mode)
        }
        Op::Comment(comment) => {
            indent(w, depth)?;
            w.write_str("ui.comment ")?;
            quoted(w, comment.content.as_str())?;
            end_line(w, comment.span, mode)
        }
        Op::If(if_op) => {
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
        Op::For(for_op) => {
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
        Op::Slot(slot) => {
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
    attributes: &[Attribute],
    bindings: &[Binding],
    children: &[Op],
    depth: usize,
    mode: Style,
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
