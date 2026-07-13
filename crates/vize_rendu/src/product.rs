//! Atlas identity for frontend-neutral render HIR.

use std::ops::Deref;

use vize_atlas::{CompilationInput, Product, SourceInput};
use vize_carton::String;

use crate::RenduRoot;

/// One frontend module's independently compilable render roots.
///
/// SFCs contribute one root. JSX/TSX can contribute several component roots,
/// but every backend still consumes this same frontend-neutral artifact.
#[derive(Debug, Clone)]
pub struct RenduModule {
    roots: Vec<RenduRoot>,
}

impl RenduModule {
    pub fn new(roots: Vec<RenduRoot>) -> Self {
        assert!(
            !roots.is_empty(),
            "a Rendu module must retain an empty root"
        );
        Self { roots }
    }

    pub fn from_root(root: RenduRoot) -> Self {
        Self { roots: vec![root] }
    }

    pub fn roots(&self) -> &[RenduRoot] {
        &self.roots
    }

    pub fn primary(&self) -> &RenduRoot {
        &self.roots[0]
    }
}

impl Deref for RenduModule {
    type Target = RenduRoot;

    fn deref(&self) -> &Self::Target {
        self.primary()
    }
}

/// Demandable render HIR produced by any applicable frontend provider.
pub struct RenduProduct;

impl Product for RenduProduct {
    type Value = RenduModule;

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

/// JavaScript packaging mode requested from a render backend.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum RenderOutputMode {
    Function,
    #[default]
    Module,
}

/// Source-scoped output settings shared by frontend-neutral backends.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenderEmitSettings {
    pub mode: RenderOutputMode,
    pub runtime_module_name: String,
    pub runtime_global_name: String,
}

impl Default for RenderEmitSettings {
    fn default() -> Self {
        Self {
            mode: RenderOutputMode::Module,
            runtime_module_name: "vue".into(),
            runtime_global_name: "Vue".into(),
        }
    }
}

/// Open per-source backend packaging configuration.
pub struct RenderEmitSettingsInput;

impl SourceInput for RenderEmitSettingsInput {
    type Value = RenderEmitSettings;

    const NAME: &'static str = "render.emit-settings";
}
