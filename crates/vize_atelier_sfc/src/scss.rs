//! Explicit original SCSS output, variable definitions and class-name products.
use vize_l0::{SourceBlock, String};
use vize_l1::css::{
    AuthoredClasses,
    lang::{StyleLanguage, StyleObservation},
};
pub use vize_l2::lang::scss::{ScssProgram, ScssRefusal};

pub struct ScssProducts<'a> {
    block: SourceBlock<'a>,
    style: StyleObservation<'a>,
    program: Result<ScssProgram<'a>, ScssRefusal>,
}

impl<'a> ScssProducts<'a> {
    /// Preprocess a selected inline Vue style at its original file offset.
    pub fn from_style(
        style: vize_l1::container::vue::StyleView<'_, 'a>,
    ) -> Result<Self, ScssRefusal> {
        let language = style
            .attrs()
            .iter()
            .find(|attr| attr.name == "lang")
            .and_then(|attr| attr.value);
        if vize_l1::container::lang::style_lang(language) != Ok(StyleLanguage::Scss) {
            return Err(ScssRefusal::Language);
        }
        Ok(Self::read(style.block()))
    }

    pub fn read(block: SourceBlock<'a>) -> Self {
        let (style, program) =
            StyleObservation::read_with_product(StyleLanguage::Scss, block, ScssProgram::lower);
        Self {
            block,
            style,
            program,
        }
    }

    /// Original owner and file-absolute extent for all projected ranges.
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    pub fn program(&self) -> Result<&ScssProgram<'a>, ScssRefusal> {
        self.program.as_ref().map_err(|error| *error)
    }

    pub fn into_parts(self) -> (StyleObservation<'a>, Result<ScssProgram<'a>, ScssRefusal>) {
        (self.style, self.program)
    }

    pub fn css(&self) -> Result<String, ScssRefusal> {
        self.program().map(vize_l4::targets::scss::emit)
    }

    pub fn classes(&self) -> Option<&AuthoredClasses> {
        self.style.authored_classes()
    }
}

#[cfg(feature = "verified-scss-edits")]
mod equivalent;
#[cfg(feature = "verified-scss-edits")]
pub use equivalent::{ScssEquivalence, ScssMismatch};
