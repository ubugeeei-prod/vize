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
pub struct Operand(pub OperandRow);

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
                    Operand((
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
                    ))
                })
                .collect(),
        }
    }
}

impl DumpValue for Operand {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let text = serde_json::to_string(&self.0).map_err(|_| fmt::Error)?;
        w.write_str("operand=")?;
        w.write_str(&text)
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, DumpError> {
        let text = text
            .strip_prefix("operand=")
            .ok_or_else(|| DumpError::new(line, cstr!("L3 value row must start with operand=")))?;
        let row: OperandRow = serde_json::from_str(text)
            .map_err(|error| DumpError::new(line, cstr!("invalid L3 operand: {error}")))?;
        let role = OperandRole::parse(&row.1)
            .ok_or_else(|| DumpError::new(line, cstr!("unknown L3 operand role")))?;
        let kind = ValueKind::parse(&row.5)
            .ok_or_else(|| DumpError::new(line, cstr!("unknown L3 value kind")))?;
        let value = OperandValue {
            kind,
            text: &row.6,
            qualifier: &row.7,
            span: Span::new(row.8, row.9),
        };
        if !value.is_well_formed() {
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
        Ok(Self(row))
    }
}
