//! Atlas identity for frontend-neutral render HIR.

use vize_atlas::{CompilationInput, Product};

use crate::RenduRoot;

/// Demandable render HIR produced by any applicable frontend provider.
pub struct RenduProduct;

impl Product for RenduProduct {
    type Value = RenduRoot;

    const NAME: &'static str = "rendu.hir";
}

/// Render capabilities that affect target planning without becoming products.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenderCapabilities {
    pub dom: bool,
    pub vapor: bool,
    pub ssr: bool,
    pub custom_renderer: bool,
}

impl Default for RenderCapabilities {
    fn default() -> Self {
        Self {
            dom: true,
            vapor: false,
            ssr: false,
            custom_renderer: false,
        }
    }
}

/// Open typed query-context dimension for render capability decisions.
pub struct RenderCapabilitiesInput;

impl CompilationInput for RenderCapabilitiesInput {
    type Value = RenderCapabilities;

    const NAME: &'static str = "render.capabilities";
}
