//! Actual source language and configured server role, never filename language.
#[cfg(feature = "experimental-source-navigation")]
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::embed::syntax::ProgramOptions;
#[cfg(feature = "experimental-source-navigation")]
use vize_l1::{container::vue::DescriptorOptions, markup::parse::SurfaceParseOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Profile {
    Program(ProgramOptions),
    #[cfg(feature = "experimental-source-navigation")]
    Vue(VueConfiguration),
    #[cfg(feature = "experimental-source-navigation")]
    SelectedVue(VueConfiguration),
    #[cfg(feature = "experimental-source-navigation")]
    TemplateNamesVue(VueConfiguration),
}

/// Coarse actual server settings; these confer no SFC or File admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "experimental-source-navigation")]
pub(in crate::source_project) struct VueConfiguration {
    pub(in crate::source_project) version: VueVersion,
    pub(in crate::source_project) configured_dialect: Option<VueDialect>,
    pub(in crate::source_project) legacy: bool,
    pub(in crate::source_project) patterned: bool,
}

#[cfg(feature = "experimental-source-navigation")]
impl VueConfiguration {
    pub(super) fn supported(self) -> bool {
        !self.patterned && !(self.legacy && self.version == VueVersion::V3)
    }

    pub(super) fn options(self) -> DescriptorOptions {
        DescriptorOptions {
            version: self.version,
            // An editor-open SFC's actual role uses Vue; standalone HTML is
            // refused before this point. Raw overrides remain freshness keys.
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum QueryFamily {
    Program,
    Selected,
    #[cfg(all(test, feature = "experimental-source-navigation"))]
    Names,
}
