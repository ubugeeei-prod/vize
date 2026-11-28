//! Read the checker expression document and editor script exports from a retained Vue File.
use super::{TemplateExpression, VirtualCodeGenerator};
use crate::virtual_code::{ExpressionKind, SourceRange, VirtualDocument};
use std::collections::BTreeSet;
use vize_atelier_sfc::SfcDescriptor;
use vize_canon::projection::native_template::{EmitDocument, project_authored_template_document};
use vize_l0::{SourceBlock, cstr};
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l2::file::Namespace;
use vize_l2::query::{TemplateOperandKind, TemplateQueryError};

pub struct TemplateOperandProducts {
    /// Canon's expression projection with file-absolute authored links.
    pub checker: EmitDocument,
    /// Maestro's original template-relative script-export inputs.
    pub expressions: Vec<TemplateExpression>,
    /// Complete editor virtual script with its ordinary URI and mappings.
    pub script_setup: Option<VirtualDocument>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateProductRefusal {
    Descriptor,
    Query(TemplateQueryError),
}

impl VirtualCodeGenerator {
    /// The retained File and the editor descriptor must describe the same
    /// source and selections before reusing the existing script emitter.
    pub fn template_operand_products(
        &mut self,
        native: &NativeSfc<'_, '_>,
        descriptor: &SfcDescriptor<'_>,
        base_uri: &str,
    ) -> Result<TemplateOperandProducts, TemplateProductRefusal> {
        let original = native.observation().l1ir();
        let selected = original
            .admitted()
            .map_err(|_| TemplateProductRefusal::Descriptor)?;
        let root = original.source();
        let editor = descriptor.source.as_ref();
        let block = selected.template().map(|t| t.block());
        if !(core::ptr::eq(root, editor) || root == editor)
            || !matches_block(
                block,
                descriptor
                    .template
                    .as_ref()
                    .map(|t| (t.content.as_ref(), t.loc.start, t.loc.end)),
            )
            || !matches_block(
                selected.setup().map(|s| s.block()),
                descriptor
                    .script_setup
                    .as_ref()
                    .map(|s| (s.content.as_ref(), s.loc.start, s.loc.end)),
            )
        {
            return Err(TemplateProductRefusal::Descriptor);
        }
        self.native_template_operand_products(native, base_uri)
    }

    /// Project the checker document and editor setup script from one admitted
    /// original File. Setup exports read its sealed lexical references; neither
    /// the script nor its template expressions are parsed again. A query refusal
    /// discards every prepared product. This explicit entry point does not select
    /// the production editor backend.
    pub fn native_template_operand_products(
        &mut self,
        native: &NativeSfc<'_, '_>,
        base_uri: &str,
    ) -> Result<TemplateOperandProducts, TemplateProductRefusal> {
        let selected = native
            .observation()
            .l1ir()
            .admitted()
            .map_err(|_| TemplateProductRefusal::Descriptor)?;
        let setup = selected.setup();
        let setup_unit = setup
            .map(|script| {
                native
                    .file()
                    .l2ir()
                    .units()
                    .iter()
                    .find(|unit| unit.id.index() as usize == script.container_index())
                    .ok_or(TemplateProductRefusal::Descriptor)
            })
            .transpose()?;
        let block = selected.template().map(|template| template.block());
        let base = block.map_or(0, SourceBlock::start);
        let mut exports = BTreeSet::new();
        let mut references = Ok(());
        let mut expressions = Vec::with_capacity(
            native
                .observation()
                .template()
                .map_or(0, |t| t.embeds().len()),
        );
        let checker = project_authored_template_document(native, |operand| {
            if references.is_ok() {
                references = operand.for_each_reference(|reference| {
                    if let (Some(unit), Some(declaration)) =
                        (setup_unit, reference.binding().declaration())
                        && declaration.unit == unit.id
                        && declaration.scope == unit.scope
                        && declaration.namespace == Namespace::Value
                    {
                        exports.insert(declaration.name.as_str());
                    }
                });
            }
            let span = operand.span();
            let kind = match operand.kind() {
                TemplateOperandKind::Interpolation => ExpressionKind::Interpolation,
                TemplateOperandKind::Bind => ExpressionKind::VBind,
                TemplateOperandKind::Condition => ExpressionKind::VIf,
                TemplateOperandKind::Model => ExpressionKind::VModel,
                TemplateOperandKind::EventReference => ExpressionKind::VOn,
            };
            expressions.push(TemplateExpression {
                kind,
                range: SourceRange::new(span.start - base, span.end - base),
                text: operand.text().to_owned(),
            });
        })
        .map_err(TemplateProductRefusal::Query)?;
        references.map_err(TemplateProductRefusal::Query)?;
        let bindings = exports.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let script_setup = setup.map(|script| {
            let mut document = self
                .script_gen
                .generate_original_setup_with_exports(script.block(), &bindings);
            document.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            document
        });
        Ok(TemplateOperandProducts {
            checker,
            expressions,
            script_setup,
        })
    }
}

fn matches_block(selected: Option<SourceBlock<'_>>, editor: Option<(&str, usize, usize)>) -> bool {
    match (selected, editor) {
        (Some(block), Some((text, start, end))) => {
            start == block.start() as usize
                && end == block.end() as usize
                && (core::ptr::eq(block.source(), text) || block.source() == text)
        }
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
