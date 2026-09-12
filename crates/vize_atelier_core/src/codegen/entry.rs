//! Public VDOM codegen entrypoints.

use crate::{
    RootNode,
    options::{CodegenExperimentalOptions, CodegenOptions},
};

use super::context::{CodegenResult, CodegenResultWithSections};
use super::emit::generate_with_sections_and_options;

/// Generate code from root AST.
pub fn generate(root: &RootNode<'_>, options: CodegenOptions) -> CodegenResult {
    generate_with_sections_and_options(
        root,
        options,
        None,
        true,
        None,
        CodegenExperimentalOptions::default(),
    )
    .into_result()
}

/// Generate code with an explicit `mergeProps` policy.
///
/// This is an additive internal integration point for JSX Babel compatibility.
/// Template compilation and existing callers continue through [`generate`],
/// which always preserves Vue template compiler `mergeProps` behavior.
#[doc(hidden)]
pub fn generate_with_merge_props(
    root: &RootNode<'_>,
    options: CodegenOptions,
    merge_props: bool,
) -> CodegenResult {
    generate_with_sections_and_options(
        root,
        options,
        None,
        merge_props,
        None,
        CodegenExperimentalOptions::default(),
    )
    .into_result()
}

/// Generate code while routing vnode creation through a caller-provided JSX
/// factory expression instead of Vue's vnode/block helpers.
pub fn generate_with_vnode_factory(
    root: &RootNode<'_>,
    options: CodegenOptions,
    vnode_factory: &str,
) -> CodegenResult {
    generate_with_sections_and_options(
        root,
        options,
        Some(vnode_factory),
        true,
        None,
        CodegenExperimentalOptions::default(),
    )
    .into_result()
}

/// Generate code with optional custom vnode creation and `mergeProps` policy.
///
/// `source_text` overrides the loc-span slicing basis; JSX roots pass the
/// module source their node spans index into (`RootNode::source` keeps the
/// root element's slice for the source-map path).
#[doc(hidden)]
pub fn generate_with_vnode_factory_and_merge_props(
    root: &RootNode<'_>,
    options: CodegenOptions,
    vnode_factory: Option<&str>,
    merge_props: bool,
    source_text: Option<&str>,
) -> CodegenResult {
    generate_with_sections_and_options(
        root,
        options,
        vnode_factory,
        merge_props,
        source_text,
        CodegenExperimentalOptions::default(),
    )
    .into_result()
}

/// Generate code from root AST and return emission-recorded section boundaries.
pub fn generate_with_sections(
    root: &RootNode<'_>,
    options: CodegenOptions,
) -> CodegenResultWithSections {
    generate_with_sections_and_options(
        root,
        options,
        None,
        true,
        None,
        CodegenExperimentalOptions::default(),
    )
}

/// Generate code with opt-in experimental codegen context.
#[doc(hidden)]
pub fn generate_with_experimental_options(
    root: &RootNode<'_>,
    options: CodegenOptions,
    experimental_options: CodegenExperimentalOptions,
) -> CodegenResult {
    generate_with_sections_and_options(root, options, None, true, None, experimental_options)
        .into_result()
}

/// Generate code and section metadata with opt-in experimental codegen context.
#[doc(hidden)]
pub fn generate_with_sections_and_experimental_options(
    root: &RootNode<'_>,
    options: CodegenOptions,
    experimental_options: CodegenExperimentalOptions,
) -> CodegenResultWithSections {
    generate_with_sections_and_options(root, options, None, true, None, experimental_options)
}
