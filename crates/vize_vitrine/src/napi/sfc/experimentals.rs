use super::types::{BatchCompileOptionsNapi, SfcCompileOptionsNapi};

#[derive(Clone, Copy)]
pub(super) struct ExperimentalTemplateOptions {
    pub(super) in_tag_comments: bool,
    pub(super) patterned_template: bool,
    pub(super) self_component: bool,
    pub(super) strict_slot_children: bool,
}

impl ExperimentalTemplateOptions {
    pub(super) fn from_batch(opts: &BatchCompileOptionsNapi) -> Self {
        Self {
            in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            patterned_template: opts.experimental_patterned_template.unwrap_or(false),
            self_component: opts.experimental_self_component.unwrap_or(false),
            strict_slot_children: opts.experimental_strict_slot_children.unwrap_or(false),
        }
    }

    pub(super) fn from_compile(opts: &SfcCompileOptionsNapi) -> Self {
        Self {
            in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            patterned_template: opts.experimental_patterned_template.unwrap_or(false),
            self_component: opts.experimental_self_component.unwrap_or(false),
            strict_slot_children: opts.experimental_strict_slot_children.unwrap_or(false),
        }
    }

    pub(super) fn bits(self) -> u16 {
        (u16::from(self.in_tag_comments) << 6)
            | (u16::from(self.patterned_template) << 7)
            | (u16::from(self.self_component) << 8)
            | (u16::from(self.strict_slot_children) << 9)
    }

    pub(super) fn dom_options(self) -> vize_atelier_dom::DomCompilerOptions {
        vize_atelier_dom::DomCompilerOptions {
            experimental_in_tag_comments: self.in_tag_comments,
            experimental_patterned_template: self.patterned_template,
            ..Default::default()
        }
    }

    pub(super) fn sfc_options(self) -> vize_atelier_sfc::SfcCompileExperimentalOptions {
        vize_atelier_sfc::SfcCompileExperimentalOptions {
            self_component: self.self_component,
        }
    }
}
