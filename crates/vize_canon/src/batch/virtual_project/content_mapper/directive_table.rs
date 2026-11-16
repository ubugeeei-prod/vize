use std::ops::Range;

use vize_atelier_sfc::SfcDescriptor;

use super::super::vue_codegen::generic_attribute::GenericCopies;
use super::directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_loose_library_prop_fallthrough,
};
use super::protocol::{ContentMapperDiagnosticDirectives, ContentMapperSpan};
use crate::virtual_ts::{ProjectionMeta, VizeMapping, diagnostic_ownership::DiagnosticEcho};

#[expect(
    clippy::too_many_arguments,
    reason = "one projection owns this complete table"
)]
pub(super) fn content_mapper_diagnostic_directives(
    source: &str,
    generated: &str,
    descriptor: &SfcDescriptor<'_>,
    spans: &[ContentMapperSpan],
    echoes: &[DiagnosticEcho],
    primaries: &[Range<usize>],
    generic_copies: Option<&GenericCopies>,
    mappings: &[VizeMapping],
    metadata: &[ProjectionMeta],
    callback_primaries: &[Range<usize>],
    loose_unknown_props: bool,
) -> Option<ContentMapperDiagnosticDirectives> {
    let directives = with_generic_echoes(
        sfc_diagnostic_directives(source, generated, descriptor, spans, echoes, primaries),
        generic_copies,
    );
    let directives =
        with_component_prop_echoes(directives, generated, mappings, callback_primaries);
    with_loose_library_prop_fallthrough(
        directives,
        source,
        generated,
        mappings,
        metadata,
        loose_unknown_props,
    )
}
