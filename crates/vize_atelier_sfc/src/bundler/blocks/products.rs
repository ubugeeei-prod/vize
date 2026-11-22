//! Project build-tool block metadata from the same L1 descriptor an editor or compiler owns.
use super::{
    BundlerCustomBlock, BundlerStyleBlock, SfcSrcInfo, custom_block_to_bundler, parse_descriptor,
    style_block_to_bundler,
};
use crate::SfcDescriptor;
use vize_l0::String;

/// The four native block-metadata answers used by build integrations.
#[derive(Debug, PartialEq, Eq)]
pub struct BundlerBlockProducts {
    /// Styles in file order, including authored content and module names.
    pub styles: Vec<BundlerStyleBlock>,
    /// Custom blocks in file order, with each block's sorted attributes.
    pub custom_blocks: Vec<BundlerCustomBlock>,
    /// External script and template selections.
    pub src: SfcSrcInfo,
    /// Whether any stylesheet is scoped.
    pub has_scoped_style: bool,
}

impl BundlerBlockProducts {
    /// Read every block product in one parse. A rejected descriptor yields
    /// the same four empty answers as the individual public operations.
    pub fn read(source: &str, filename: Option<&str>) -> Self {
        Self::project(parse_descriptor(source, filename).as_ref())
    }

    /// Consume an already-parsed original descriptor without another source walk.
    pub fn from_descriptor(descriptor: &SfcDescriptor<'_>) -> Self {
        Self::project(Some(descriptor))
    }

    fn project(descriptor: Option<&SfcDescriptor<'_>>) -> Self {
        let mut has_scoped_style = false;
        let styles = descriptor
            .map_or(&[][..], |d| d.styles.as_slice())
            .iter()
            .enumerate()
            .map(|row| {
                has_scoped_style |= row.1.scoped;
                style_block_to_bundler(row)
            })
            .collect();
        let custom_blocks = descriptor
            .map_or(&[][..], |d| d.custom_blocks.as_slice())
            .iter()
            .enumerate()
            .map(custom_block_to_bundler)
            .collect();
        let script = descriptor.and_then(|d| d.script.as_ref().or(d.script_setup.as_ref()));
        let template = descriptor.and_then(|d| d.template.as_ref());
        let src = SfcSrcInfo {
            script_src: script.and_then(|s| s.src.as_deref()).map(String::from),
            template_src: template.and_then(|t| t.src.as_deref()).map(String::from),
        };
        Self {
            styles,
            custom_blocks,
            src,
            has_scoped_style,
        }
    }
}
