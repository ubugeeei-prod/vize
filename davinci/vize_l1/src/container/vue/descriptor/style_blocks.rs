//! A style consumer needs trustworthy block boundaries and attributes. The
//! syntax or source selection of a sibling script or template is independent.
use super::{
    AdmittedDescriptor, DescriptorIssue, DescriptorIssueCode as Code, DescriptorObservation,
    DescriptorRefusal, StyleView,
};
use crate::container::ContainerErrorCode;

/// Original style selections in authored order, with their descriptor and source.
#[derive(Debug, Clone, Copy)]
pub struct CheckedStyleBlocks<'o, 'a> {
    owner: &'o DescriptorObservation<'a>,
}

impl<'o, 'a> From<AdmittedDescriptor<'o, 'a>> for CheckedStyleBlocks<'o, 'a> {
    fn from(descriptor: AdmittedDescriptor<'o, 'a>) -> Self {
        Self {
            owner: descriptor.owner,
        }
    }
}

impl<'o, 'a> CheckedStyleBlocks<'o, 'a> {
    pub fn source(self) -> &'a str {
        self.owner.source()
    }
    pub fn styles(self) -> impl ExactSizeIterator<Item = StyleView<'o, 'a>> {
        self.owner
            .styles
            .iter()
            .copied()
            .map(move |selected| StyleView {
                owner: self.owner,
                selected,
            })
    }
}

impl<'a> DescriptorObservation<'a> {
    /// Check the independently selected styles. A duplicate script or template
    /// has a trustworthy envelope. All framing errors invalidate the view.
    pub fn admitted_styles(&self) -> Result<CheckedStyleBlocks<'_, 'a>, DescriptorRefusal<'_>> {
        let independent = |issue: &DescriptorIssue| {
            if issue.code == Code::MissingComponentBlock {
                return true;
            }
            let owner = issue
                .container_index
                .and_then(|i| self.container.blocks.get(i));
            matches!(owner, Some(block) if matches!(block.name, "script" | "template"))
                && matches!(
                    issue.code,
                    Code::ExternalSource
                        | Code::UnsupportedLanguage
                        | Code::EncodedLanguage
                        | Code::UnsupportedSetupValue
                        | Code::LanguageMismatch
                        | Code::DuplicateRole
                        | Code::UnsupportedScript
                )
        };
        if self.root.is_some()
            && self
                .container
                .errors
                .iter()
                .all(|error| error.code == ContainerErrorCode::DuplicateBlock)
            && self.issues.iter().all(independent)
        {
            Ok(CheckedStyleBlocks { owner: self })
        } else {
            Err(DescriptorRefusal {
                issues: &self.issues,
                errors: &self.container.errors,
            })
        }
    }
}
