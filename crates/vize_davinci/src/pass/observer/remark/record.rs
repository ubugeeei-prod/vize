//! The owned remark record and the collecting observer.
//!
//! [`RemarkCollector`] is the one observer that stores remark text; it is
//! what every serialized form (the `[remarks]` folio page, the JSON
//! document, the Spolvero feed) is produced from, so they cannot disagree.
//!
//! # Canonical order
//!
//! [`RemarkCollector::finish`] orders remarks by the run position of the
//! emitting pass, then by span start ascending, span end descending (an
//! outer construct before the inner one it contains), then by remark name;
//! remaining ties keep emission order. The order is therefore independent
//! of a pass's walk order - a post-order analysis and a pre-order one that
//! decide the same things produce the same log - which is what lets a
//! corpus remarks-diff read a changed line as a changed decision.

use alloc::vec::Vec;
use core::cmp::Reverse;

use vize_s0::{Span, String};

use super::{PassEvent, PassObserver, Remark, RemarkKind, RemarkValue};

/// An owned argument value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemarkArgValue {
    /// Text.
    Str(String),
    /// An integer.
    Int(i64),
    /// A boolean.
    Bool(bool),
}

impl From<RemarkValue<'_>> for RemarkArgValue {
    fn from(value: RemarkValue<'_>) -> Self {
        match value {
            RemarkValue::Str(text) => Self::Str(String::from(text)),
            RemarkValue::Int(number) => Self::Int(number),
            RemarkValue::Bool(flag) => Self::Bool(flag),
        }
    }
}

/// One owned `key=value` argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedArg {
    /// The argument's name.
    pub key: String,
    /// Its value.
    pub value: RemarkArgValue,
}

/// A remark as recorded: the emitted [`Remark`] plus the stage and pass the
/// channel attributed it to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRemark {
    /// The emitting pipeline's stage (`s2`).
    pub stage: String,
    /// The emitting pass (`hoist-static`).
    pub pass: String,
    /// Applied, missed, or analysis.
    pub kind: RemarkKind,
    /// The remark's identity within its pass.
    pub name: String,
    /// The authored span the decision is about.
    pub span: Span,
    /// Structured arguments, in emission order.
    pub args: Vec<RecordedArg>,
}

impl RecordedRemark {
    /// Record `remark` as emitted by `event`'s pass.
    #[must_use]
    pub fn of(event: &PassEvent<'_>, remark: &Remark<'_>) -> Self {
        Self {
            stage: String::from(event.pipeline.stage),
            pass: String::from(event.desc().name),
            kind: remark.kind,
            name: String::from(remark.name),
            span: remark.span,
            args: remark
                .args
                .iter()
                .map(|arg| RecordedArg {
                    key: String::from(arg.key),
                    value: RemarkArgValue::from(arg.value),
                })
                .collect(),
        }
    }

    /// The text value of argument `key`, if present and textual.
    #[must_use]
    pub fn arg_str(&self, key: &str) -> Option<&str> {
        self.args.iter().find_map(|arg| match &arg.value {
            RemarkArgValue::Str(text) if arg.key.as_str() == key => Some(text.as_str()),
            _ => None,
        })
    }
}

/// Records every remark a run emits, for serialization.
///
/// # Panics
///
/// [`on_remark`](PassObserver::on_remark) panics on a remark repeating an
/// argument key: a duplicate key is a pass bug, and every serialized form
/// keys arguments by name.
#[derive(Debug, Default)]
pub struct RemarkCollector {
    /// Run position of the pass currently executing (1-based; 0 before the
    /// first pass).
    pass_seq: u32,
    /// `(pass run position, remark)` in emission order.
    remarks: Vec<(u32, RecordedRemark)>,
}

impl RemarkCollector {
    /// An empty collector.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pass_seq: 0,
            remarks: Vec::new(),
        }
    }

    /// How many remarks were recorded.
    #[must_use]
    pub fn len(&self) -> usize {
        self.remarks.len()
    }

    /// Whether nothing was recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remarks.is_empty()
    }

    /// The recorded remarks in canonical order (module docs).
    #[must_use]
    pub fn finish(mut self) -> Vec<RecordedRemark> {
        // `sort_by` is stable: full ties keep emission order.
        self.remarks.sort_by(|(a_seq, a), (b_seq, b)| {
            (a_seq, a.span.start, Reverse(a.span.end), a.name.as_str()).cmp(&(
                b_seq,
                b.span.start,
                Reverse(b.span.end),
                b.name.as_str(),
            ))
        });
        self.remarks.into_iter().map(|(_, remark)| remark).collect()
    }
}

impl PassObserver for RemarkCollector {
    const REMARKS: bool = true;

    fn before_pass(&mut self, _event: &PassEvent<'_>) {
        self.pass_seq += 1;
    }

    fn on_remark(&mut self, event: &PassEvent<'_>, remark: &Remark<'_>) {
        for (index, arg) in remark.args.iter().enumerate() {
            assert!(
                remark.args[..index].iter().all(|seen| seen.key != arg.key),
                "remark `{}` from pass `{}` repeats argument `{}`",
                remark.name,
                event.desc().name,
                arg.key
            );
        }
        self.remarks
            .push((self.pass_seq, RecordedRemark::of(event, remark)));
    }
}
