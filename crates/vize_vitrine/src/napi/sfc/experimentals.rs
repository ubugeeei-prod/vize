use super::types::{BatchCompileOptionsNapi, SfcCompileOptionsNapi};

#[derive(Clone, Copy)]
pub(super) struct ExperimentalTemplateOptions {
    pub(super) in_tag_comments: bool,
    pub(super) patterned_template: bool,
}

impl ExperimentalTemplateOptions {
    pub(super) fn from_batch(opts: &BatchCompileOptionsNapi) -> Self {
        Self {
            in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            patterned_template: opts.experimental_patterned_template.unwrap_or(false),
        }
    }

    pub(super) fn from_compile(opts: &SfcCompileOptionsNapi) -> Self {
        Self {
            in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            patterned_template: opts.experimental_patterned_template.unwrap_or(false),
        }
    }

    pub(super) fn bits(self) -> u16 {
        (u16::from(self.in_tag_comments) << 6) | (u16::from(self.patterned_template) << 7)
    }

    pub(super) fn dom_options(self) -> vize_atelier_dom::DomCompilerOptions {
        vize_atelier_dom::DomCompilerOptions {
            experimental_in_tag_comments: self.in_tag_comments,
            experimental_patterned_template: self.patterned_template,
            ..Default::default()
        }
    }
}
