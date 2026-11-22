//! Emit the checker expression document while another reader observes those same operands.
pub use vize_atelier_core::codegen::document::EmitDocument;
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::query::{ResolvedTemplateOperand, TemplateQueryError};

/// Emit decoded expression text and its complete authored extent. A refused
/// visit discards the document and invalidates any caller observations.
pub fn project_authored_template_document<'o, 'a>(
    native: &NativeSfc<'o, 'a>,
    mut observe: impl FnMut(ResolvedTemplateOperand<'o, 'a>),
) -> Result<EmitDocument, TemplateQueryError> {
    let mut document = EmitDocument::new(true);
    native.for_each_authored_template_operand(|operand| {
        if !document.is_empty() {
            document.push_str("\n");
        }
        document.push_linked(operand.text(), operand.span());
        observe(operand);
    })?;
    Ok(document)
}
