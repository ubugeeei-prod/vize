//! Source-map bridging for the S2 DOM production selector.
//!
//! The S2 DOM emitter owns the selected render code. Until S4 carries mapping
//! spans through a structured emitter, source-map requests borrow the existing
//! compatibility map only after the compatibility generator proves it would
//! have emitted the same render module bytes and section boundaries.

use vize_atelier_core::{
    RootNode,
    codegen::{CodegenResultWithSections, generate_with_sections},
    options::CodegenOptions,
};
use vize_s0::profile;

pub(super) fn attach_compat_map(
    root: &RootNode<'_>,
    codegen_options: &CodegenOptions,
    mut s2: CodegenResultWithSections,
) -> CodegenResultWithSections {
    if !codegen_options.source_map {
        return s2;
    }

    let compat = profile!(
        "atelier.dom.template.codegen_sourcemap_compat",
        generate_with_sections(root, codegen_options.clone())
    );
    if same_generated_output(&s2, &compat) {
        s2.result.map = compat.result.map;
        s2
    } else {
        compat
    }
}

fn same_generated_output(a: &CodegenResultWithSections, b: &CodegenResultWithSections) -> bool {
    a.result.preamble == b.result.preamble
        && a.result.code == b.result.code
        && a.sections == b.sections
}
