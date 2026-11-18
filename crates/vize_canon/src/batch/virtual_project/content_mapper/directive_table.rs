use std::ops::Range;

use vize_atelier_sfc::SfcDescriptor;

use super::super::vue_codegen::generic_attribute::GenericCopies;
use super::directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_loose_library_prop_fallthrough,
};
use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
};
use crate::authored_sfc::AuthoredSfc;
use crate::virtual_ts::{ProjectionMeta, VizeMapping, diagnostic_ownership::DiagnosticEcho};

/// The transform's directive table, before owned-input echoes.
///
/// Directive comments are read from HTML templates and SFC blocks only. The
/// echo passes read the analyzed SFC and its unmapped `mappings`: a Pug
/// template's derived Vue template has the HTML shape there, so both get the
/// same echoes (#13284), and their owners map back to the Pug source.
#[expect(
    clippy::too_many_arguments,
    reason = "one projection owns this complete table"
)]
pub(super) fn content_mapper_diagnostic_directives(
    authored: &AuthoredSfc<'_>,
    descriptor: &SfcDescriptor<'_>,
    generated: &str,
    spans: &[ContentMapperSpan],
    echoes: &[DiagnosticEcho],
    primaries: &[Range<usize>],
    generic_copies: Option<&GenericCopies>,
    mappings: &[VizeMapping],
    metadata: &[ProjectionMeta],
    callback_primaries: &[Range<usize>],
    loose_unknown_props: bool,
) -> Option<ContentMapperDiagnosticDirectives> {
    let source = authored.analyzed();
    let derived = authored.is_derived();
    let comments = (!derived)
        .then(|| sfc_diagnostic_directives(source, generated, descriptor, spans, echoes, primaries))
        .flatten();
    let directives = with_generic_echoes(comments, generic_copies);
    let directives =
        with_component_prop_echoes(directives, generated, mappings, callback_primaries);
    let mut directives = with_loose_library_prop_fallthrough(
        directives,
        source,
        generated,
        mappings,
        metadata,
        loose_unknown_props,
    );
    // Only a derived template has analyzed owners, and it has no comment rows.
    if derived {
        for ContentMapperDiagnosticDirective([start, length, ..]) in directives
            .iter_mut()
            .flat_map(|table| &mut table.directives)
        {
            let owner = authored.to_authored(*start..*start + *length);
            (*start, *length) = (owner.start, owner.len());
        }
    }
    directives
}
