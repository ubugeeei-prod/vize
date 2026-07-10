//! Atlas identity for the owned, source-faithful Relief syntax product.

use vize_atlas::{CompilationInput, Product};
use vize_carton::config::VueVersion;

use crate::ReliefSnapshot;

/// Demandable Vue-template syntax snapshot.
///
/// Parsing and lowering providers live in frontend crates. Relief owns only
/// the value type and its open graph identity.
pub struct ReliefProduct;

impl Product for ReliefProduct {
    type Value = ReliefSnapshot;

    const NAME: &'static str = "relief.syntax";
}

/// Vue language line relevant to syntax and semantic providers.
pub struct VueDialectInput;

impl CompilationInput for VueDialectInput {
    type Value = VueVersion;

    const NAME: &'static str = "vue.dialect";
}
