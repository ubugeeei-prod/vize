//! Per-binding type products from verified original TS initializer judgments.
use crate::{SfcTypeDiagnostic, SfcTypeSeverity};
use oxc_ast::ast::VariableDeclarationKind;
use vize_l0::{Span, cstr};
use vize_l2::query::script::assignments::{Assignment, CheckedAssignments, Primitive, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingType {
    pub binding: Span,
    pub kind: VariableDeclarationKind,
    pub exported: bool,
    pub declared: Primitive,
    pub initializer: Span,
    pub literal: Primitive,
}

impl From<&Assignment> for BindingType {
    fn from(row: &Assignment) -> Self {
        Self {
            binding: row.binding,
            kind: row.kind,
            exported: row.exported,
            declared: row.expected,
            initializer: row.initializer,
            literal: row.actual,
        }
    }
}

/// In source order, report the explicit type of each binding and the TS2322
/// diagnostic for a mismatched initializer in the supported fragment.
pub fn visit_binding_types(
    checked: &CheckedAssignments<'_, '_, '_>,
    mut report: impl FnMut(BindingType, Option<SfcTypeDiagnostic>),
) {
    for row in checked.assignments() {
        let diagnostic = (row.verdict == Verdict::Mismatch).then(|| SfcTypeDiagnostic {
            start: row.binding.start,
            end: row.binding.end,
            severity: SfcTypeSeverity::Error,
            code: Some("TS2322".into()),
            message: cstr!(
                "Type '{}' is not assignable to type '{}'.",
                row.actual.keyword(),
                row.expected.keyword()
            ),
            help: None,
            related: Vec::new(),
        });
        report(BindingType::from(row), diagnostic);
    }
}
