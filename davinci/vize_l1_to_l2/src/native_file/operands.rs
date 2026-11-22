//! Join each original template expression to its File operand in authored order.
use super::NativeSfc;
use vize_l2::query::{Model, ResolvedTemplateOperand, TemplateQueryError};

impl<'o, 'a> NativeSfc<'o, 'a> {
    /// Visit operands in the original Vue template's order, including constants.
    /// A refusal invalidates the whole visit, including earlier observations.
    pub fn for_each_authored_template_operand(
        &self,
        mut visit: impl FnMut(ResolvedTemplateOperand<'o, 'a>),
    ) -> Result<(), TemplateQueryError> {
        let file = self.file().file();
        let model = Model::new(file);
        let template = self.observation().template();
        let embeds = template.map_or(&[][..], |t| t.embeds());
        if model.resolved_template_operand_count()? != embeds.len() {
            return Err(TemplateQueryError::OriginalMembership);
        }
        let Some(template) = template else {
            return Ok(());
        };
        let block = template.block();
        let mut previous_end = block.start();
        for embed in embeds {
            let node = embed.node.ok_or(TemplateQueryError::OriginalMembership)?;
            let operand = model
                .resolved_template_operand(node)?
                .ok_or(TemplateQueryError::OriginalMembership)?;
            let source = embed.syntax.source();
            let span = operand.span();
            if span != source.span()
                || span.start < previous_end
                || span.start >= span.end
                || span.end > block.end()
                || !core::ptr::eq(source.authored_root(), file.artifact().source())
                || !core::ptr::eq(source.text(), operand.text())
            {
                return Err(TemplateQueryError::OriginalMembership);
            }
            previous_end = span.end;
            visit(operand);
        }
        Ok(())
    }
}
