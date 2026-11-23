//! Structural tooling can use one unambiguous original HTML template even
//! when a sibling needs another language or an external source.
use super::{DescriptorObservation, TemplateFrameNameRefusal, TemplateView};
use crate::container::ContainerErrorCode;
use crate::container::vue::{DescriptorIssueCode as Code, DescriptorRefusal};
use crate::markup::SurfaceParseOptions;
use vize_l0::{SourceBlock, Span};

impl<'a> DescriptorObservation<'a> {
    /// Borrow the original HTML template, or report its genuine absence.
    /// A framing error anywhere in the document or an ambiguous template
    /// keeps the splitter and policy diagnostics available to the caller.
    pub fn admitted_template(
        &self,
    ) -> Result<Option<CheckedTemplateBlock<'_, 'a>>, DescriptorRefusal<'_>> {
        if let Ok(whole) = self.admitted() {
            return Ok(whole.template().map(CheckedTemplateBlock::from));
        }
        let independent = self.issues.iter().all(|issue| {
            if issue.code == Code::MissingComponentBlock {
                return self.template.is_none();
            }
            let block = issue
                .container_index
                .and_then(|i| self.container.blocks.get(i));
            match block.map(|b| b.name) {
                Some("script") => matches!(
                    issue.code,
                    Code::ExternalSource
                        | Code::UnsupportedLanguage
                        | Code::EncodedLanguage
                        | Code::UnsupportedSetupValue
                        | Code::LanguageMismatch
                        | Code::DuplicateRole
                        | Code::UnsupportedScript
                ),
                Some("style") => matches!(
                    issue.code,
                    Code::ExternalSource
                        | Code::UnsupportedLanguage
                        | Code::EncodedLanguage
                        | Code::UnsupportedStyle
                ),
                _ => false,
            }
        });
        // The descriptor records DuplicateRole on the actual duplicate block.
        // It is accepted above only for a script. Every framing error is fatal.
        if self.root.is_some()
            && independent
            && self
                .container
                .errors
                .iter()
                .all(|e| e.code == ContainerErrorCode::DuplicateBlock)
        {
            Ok(self.template.as_ref().map(|selected| CheckedTemplateBlock {
                view: TemplateView {
                    owner: self,
                    selected,
                },
            }))
        } else {
            Err(DescriptorRefusal {
                issues: &self.issues,
                errors: &self.container.errors,
            })
        }
    }
}

/// The original block and frame for structural queries. A whole-file consumer
/// still requires its original descriptor or selected-template capability.
/// ```compile_fail
/// use vize_l1::container::vue::{CheckedTemplateBlock, TemplateView};
/// fn promote<'o,'a>(block: CheckedTemplateBlock<'o,'a>) -> TemplateView<'o,'a> { block.into() }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CheckedTemplateBlock<'o, 'a> {
    view: TemplateView<'o, 'a>,
}
impl<'o, 'a> From<TemplateView<'o, 'a>> for CheckedTemplateBlock<'o, 'a> {
    fn from(view: TemplateView<'o, 'a>) -> Self {
        Self { view }
    }
}

impl<'a> CheckedTemplateBlock<'_, 'a> {
    #[must_use]
    pub fn block(self) -> SourceBlock<'a> {
        self.view.block()
    }
    #[must_use]
    pub fn container_index(self) -> usize {
        self.view.container_index()
    }
    /// Complete original opening tag, including delimiters and attributes.
    #[must_use]
    pub fn open_tag(self) -> Span {
        #[expect(
            clippy::indexing_slicing,
            reason = "the selection index was minted from this immutable descriptor's block list"
        )]
        self.view.owner.container.blocks[self.container_index()].open_tag
    }
    /// The original opening and closing tag name spans.
    pub fn tag_names(self) -> Result<(Span, Span), TemplateFrameNameRefusal> {
        let names = self.view.frame_names()?;
        Ok((names.opening(), names.closing()))
    }
    /// The descriptor's actual HTML tree construction and recovery settings.
    #[must_use]
    pub fn surface_options(self) -> SurfaceParseOptions {
        self.view.owner.options().template
    }
}
