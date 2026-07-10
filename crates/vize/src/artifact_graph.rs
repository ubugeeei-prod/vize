//! Application-level registration of independent artifact providers.
//!
//! This module assembles providers; it does not orchestrate parse/analyze/lower
//! calls. Tool recipes request root products and Atlas derives the closure.

use std::{error::Error, fmt};

use vize_atlas::{Compilation, CompilationInputError, ProductId, RegisterProviderError};
use vize_carton::config::VueVersion;
use vize_relief::VueDialectInput;
use vize_rendu::{RenderCapabilities, RenderCapabilitiesInput};

/// Open query-context values installed for a Vize compilation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VizeGraphConfig {
    pub vue: VueVersion,
    pub render: RenderCapabilities,
}

impl Default for VizeGraphConfig {
    fn default() -> Self {
        Self {
            vue: VueVersion::V3,
            render: RenderCapabilities {
                dom: true,
                ssr: true,
                vapor: true,
                custom_renderer: false,
            },
        }
    }
}

/// Failure while assembling the application provider registry.
#[derive(Debug)]
pub enum VizeGraphError {
    Register(RegisterProviderError),
    Input(CompilationInputError),
}

impl fmt::Display for VizeGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(error) => error.fmt(formatter),
            Self::Input(error) => error.fmt(formatter),
        }
    }
}

impl Error for VizeGraphError {}

impl From<RegisterProviderError> for VizeGraphError {
    fn from(error: RegisterProviderError) -> Self {
        Self::Register(error)
    }
}

impl From<CompilationInputError> for VizeGraphError {
    fn from(error: CompilationInputError) -> Self {
        Self::Input(error)
    }
}

/// Build a compilation with equal SFC/JSX frontends, peer representations,
/// target backends, and Patina/Canon recipes registered.
pub fn create_compilation(config: VizeGraphConfig) -> Result<Compilation, VizeGraphError> {
    let mut compilation = Compilation::new();
    register_providers(&mut compilation)?;
    compilation.set_input::<VueDialectInput>(config.vue)?;
    compilation.set_input::<RenderCapabilitiesInput>(config.render)?;
    Ok(compilation)
}

/// Register providers without allocating per-source artifact state.
pub fn register_providers(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    vize_atelier_sfc::register_atlas_providers(compilation)?;
    vize_atelier_jsx::register_atlas_providers(compilation)?;
    vize_atelier_dom::register_atlas_provider(compilation)?;
    vize_atelier_ssr::register_atlas_provider(compilation)?;
    vize_atelier_vapor::register_atlas_provider(compilation)?;
    vize_patina::register_semantic_lint_recipe(compilation)?;
    vize_canon::register_semantic_virtual_ts_recipe(compilation)?;
    vize_croquis_cf::register_atlas_provider(compilation)
}

/// Compiler recipe roots. Atlas plans all shared work once.
pub fn compiler_roots(dom: bool, ssr: bool, vapor: bool) -> Vec<ProductId> {
    let mut roots = Vec::with_capacity(3);
    if dom {
        roots.push(ProductId::of::<vize_atelier_dom::DomOutputProduct>());
    }
    if ssr {
        roots.push(ProductId::of::<vize_atelier_ssr::SsrOutputProduct>());
    }
    if vapor {
        roots.push(ProductId::of::<vize_atelier_vapor::VaporPlanProduct>());
    }
    roots
}

/// Combined lint/typecheck recipe roots sharing one semantic query.
pub fn analysis_roots(lint: bool, typecheck: bool) -> Vec<ProductId> {
    let mut roots = Vec::with_capacity(2);
    if lint {
        roots.push(ProductId::of::<vize_patina::PatinaSemanticReportProduct>());
    }
    if typecheck {
        roots.push(ProductId::of::<vize_canon::CanonSemanticVirtualTsProduct>());
    }
    roots
}

/// Opt-in project-analysis roots. Ordinary single-source recipes omit these.
pub fn project_roots(cross_file: bool) -> Vec<ProductId> {
    cross_file
        .then(ProductId::of::<vize_croquis_cf::CroquisProjectProduct>)
        .into_iter()
        .collect()
}
