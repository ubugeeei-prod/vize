//! Owned value page paired with the graph-only `L3Page` view.
//! JSON tuples keep arbitrary authored strings line-atomic without inventing
//! another quoting grammar. Tuple equality is document equality only.

use alloc::vec::Vec;
use core::fmt;
use vize_l0::dump::value::DumpValue;
use vize_l0::dump::{Dump, Error as DumpError};
use vize_l0::{Span, String, cstr};

use crate::op::Program;
use crate::operand::{OperandRole, OperandValue, ValueKind};

/// op, role, target, region, attribute name, kind, text, qualifier, start, end.
pub type OperandRow = (
    u32,
    String,
    Option<u32>,
    Option<u32>,
    Option<String>,
    String,
    String,
    String,
    u32,
    u32,
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operand(pub OperandRow, pub Option<u32>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Dump)]
#[dump(name = "s3-values-folio")]
pub struct Page {
    pub operands: Vec<Operand>,
}

impl Page {
    #[must_use]
    pub fn of(program: &Program<'_>) -> Self {
        Self {
            operands: program
                .operands
                .iter()
                .map(|operand| {
                    Operand(
                        (
                            operand.op.index(),
                            String::from(operand.role.as_str()),
                            operand.target.map(|id| id.index()),
                            operand.region.map(|id| id.index()),
                            operand.name.map(String::from),
                            String::from(operand.value.kind.as_str()),
                            String::from(operand.value.text),
                            String::from(operand.value.qualifier),
                            operand.value.span.start,
                            operand.value.span.end,
                        ),
                        operand
                            .value
                            .native_handler
                            .map(|handler| handler.node().index()),
                    )
                })
                .collect(),
        }
    }
}

impl DumpValue for Operand {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let text = serde_json::to_string(&self.0).map_err(|_| fmt::Error)?;
        w.write_str("operand=")?;
        w.write_str(&text)?;
        if let Some(handler) = self.1 {
            write!(w, " handler-ref={handler}")?;
        }
        Ok(())
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, DumpError> {
        let text = text
            .strip_prefix("operand=")
            .ok_or_else(|| DumpError::new(line, cstr!("L3 value row must start with operand=")))?;
        // This is owned diagnostic data only. Reading a numeric field never
        // creates a sealed HandlerId or confers File/original-On membership.
        let (text, handler) = match text.rsplit_once(" handler-ref=") {
            Some((row, index))
                if !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                let index = index
                    .parse::<u32>()
                    .map_err(|_| DumpError::new(line, cstr!("invalid L3 handler reference")))?;
                (row, Some(index))
            }
            _ => (text, None),
        };
        let row: OperandRow = serde_json::from_str(text)
            .map_err(|error| DumpError::new(line, cstr!("invalid L3 operand: {error}")))?;
        let role = OperandRole::parse(&row.1)
            .ok_or_else(|| DumpError::new(line, cstr!("unknown L3 operand role")))?;
        let kind = ValueKind::parse(&row.5)
            .ok_or_else(|| DumpError::new(line, cstr!("unknown L3 value kind")))?;
        let value = OperandValue {
            kind,
            native_handler: None,
            text: &row.6,
            qualifier: &row.7,
            span: Span::new(row.8, row.9),
        };
        let native = kind == ValueKind::NativeHandler;
        if if native {
            handler != Some(row.0)
                || role != OperandRole::Value
                || !row.6.is_empty()
                || !row.7.is_empty()
                || row.8 > row.9
        } else {
            handler.is_some() || !value.is_well_formed()
        } {
            return Err(DumpError::new(line, cstr!("malformed L3 operand value")));
        }
        if !role.accepts_target(row.2.is_some())
            || row.4.is_some()
                != matches!(role, OperandRole::Attribute | OperandRole::ModelAttribute)
            || row.3.is_some() != (role == OperandRole::Condition)
            || (row.3.is_some() && row.2.is_some())
        {
            return Err(DumpError::new(
                line,
                cstr!("malformed L3 operand references"),
            ));
        }
        Ok(Self(row, handler))
    }
}
