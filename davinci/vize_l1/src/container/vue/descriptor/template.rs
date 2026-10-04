//! Original selected-template view.

use super::{DescriptorObservation, Selection};
use vize_l0::SourceBlock;

/// Checked original ordinary HTML template content.
#[derive(Debug, Clone, Copy)]
pub struct TemplateView<'o, 'a> {
    pub(super) owner: &'o DescriptorObservation<'a>,
    pub(super) selected: Selection<'a>,
}

impl<'a> TemplateView<'_, 'a> {
    pub fn source(&self) -> &'a str {
        self.owner.source()
    }
    pub fn container_index(&self) -> usize {
        self.selected.index
    }
    pub fn block(&self) -> SourceBlock<'a> {
        self.selected.block
    }
}
