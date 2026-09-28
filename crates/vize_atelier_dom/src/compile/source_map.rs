//! Source-map bridging for the L2 DOM production selector.
//!
//! The L2 DOM emitter owns the selected render code. Until L4 carries mapping
//! spans through a structured emitter, source-map requests borrow the existing
//! compatibility map only after the compatibility generator proves it would
//! have emitted the same render module bytes and section boundaries.

use vize_atelier_core::{
    RootNode,
    codegen::{CodegenResultWithSections, generate_with_sections},
    options::CodegenOptions,
};
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink};
use vize_l0::profile;

use super::selection::{self, DomLegacyReason};

/// The final code producer after the compatibility map has been checked.
pub(super) enum SourceMapSelection {
    Native(CodegenResultWithSections),
    Compatibility(CodegenResultWithSections),
}

impl SourceMapSelection {
    pub(super) fn finish<C: CaptureSink>(self, capture: &mut C) -> CodegenResultWithSections {
        match self {
            Self::Native(result) => {
                selection::record(Ok(()));
                capture.finish(|| CaptureOutcome::Accepted);
                result
            }
            Self::Compatibility(result) => {
                selection::record(Err(DomLegacyReason::SourceMapMismatch));
                capture.finish(|| {
                    CaptureOutcome::Legacy(vize_l0::String::from(
                        DomLegacyReason::SourceMapMismatch.id(),
                    ))
                });
                result
            }
        }
    }
}

pub(super) fn attach_compat_map(
    root: &RootNode<'_>,
    codegen_options: &CodegenOptions,
    s2: CodegenResultWithSections,
) -> SourceMapSelection {
    if !codegen_options.source_map {
        return SourceMapSelection::Native(s2);
    }

    let compat = profile!(
        "atelier.dom.template.codegen_sourcemap_compat",
        generate_with_sections(root, codegen_options.clone())
    );
    select_with_compat(s2, compat)
}

fn select_with_compat(
    mut native: CodegenResultWithSections,
    compat: CodegenResultWithSections,
) -> SourceMapSelection {
    if same_generated_output(&native, &compat) {
        native.result.map = compat.result.map;
        SourceMapSelection::Native(native)
    } else {
        SourceMapSelection::Compatibility(compat)
    }
}

fn same_generated_output(a: &CodegenResultWithSections, b: &CodegenResultWithSections) -> bool {
    a.result.preamble == b.result.preamble
        && a.result.code == b.result.code
        && a.sections == b.sections
}

#[cfg(test)]
mod tests {
    use super::select_with_compat;
    use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
    use vize_l0::{
        String,
        dump::capture::{CaptureOutcome, CaptureSink, StageCapture},
        level::Level,
    };

    #[test]
    fn source_map_mismatch_returns_compatibility_output_without_native_pages() {
        let result = |code: &str| CodegenResultWithSections {
            result: CodegenResult {
                code: String::from(code),
                preamble: String::default(),
                map: None,
            },
            sections: None,
        };
        let mut capture = StageCapture::new("dom");
        capture.page(Level::L1, "parse", || String::from("provisional"));
        let returned = select_with_compat(result("native"), result("compat")).finish(&mut capture);

        assert_eq!(returned.result.code, "compat");
        assert_eq!(
            capture.outcome,
            CaptureOutcome::Legacy(String::from("source-map-mismatch"))
        );
        assert!(capture.pages.is_empty());
    }
}
