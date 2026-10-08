//! Consumer policy over explicit script and template execution ownership.

use crate::diagnostics::DiagnosticSource;
use vize_croquis::{Croquis, TemplateExpressionKind};

pub(super) fn is_client_usage(
    analysis: &Croquis,
    offset: u32,
    source: DiagnosticSource,
    regions: &[(u32, u32)],
) -> bool {
    match source {
        DiagnosticSource::Unspecified => false,
        DiagnosticSource::Script => {
            regions
                .iter()
                .any(|&(start, end)| start <= offset && offset < end)
                || super::is_in_client_only_context(analysis, offset)
        }
        DiagnosticSource::Template => analysis.template_expressions.iter().any(|expression| {
            expression.kind == TemplateExpressionKind::VOn && expression.start == offset
        }),
    }
}
