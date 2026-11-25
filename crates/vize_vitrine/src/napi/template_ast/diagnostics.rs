//! Parser diagnostics as data: a recovery note is a warning, never a throw.

use vize_l1::markup::{ComponentParse, DirectiveNameError, SurfaceError};

use super::tree::Writer;

/// `,"errors":[…],"warnings":[…]`. Tree-construction recovery notes (the
/// `ExtendPoint` code) describe a repair already applied to valid input and a
/// duplicate attribute does not stop compilation, so both are warnings; every
/// other parser diagnostic is an error. Neither stops the tree from being
/// returned.
pub(super) fn write(writer: &mut Writer<'_>, parse: &ComponentParse<'_>) {
    writer.json.raw(",\"errors\":[");
    let mut first = true;
    for error in parse.errors.iter().filter(|error| !is_warning(error)) {
        surface(writer, error, &mut first);
    }
    for admission in &parse.unsupported {
        separator(writer, &mut first);
        writer
            .json
            .raw("{\"code\":\"vize/unsupported-directive-head\",\"message\":");
        writer.json.string(match admission.error {
            DirectiveNameError::OffsetOverflow => {
                "This directive name lies outside the addressable source range, so it is kept as a plain attribute."
            }
            DirectiveNameError::NestingLimit => {
                "This directive argument nests brackets deeper than the parser supports, so it is kept as a plain attribute."
            }
        });
        writer.json.raw(",\"loc\":");
        writer.loc(admission.span.start, admission.span.end);
        writer.json.raw("}");
    }
    writer.json.raw("],\"warnings\":[");
    let mut first = true;
    for error in parse.errors.iter().filter(|error| is_warning(error)) {
        surface(writer, error, &mut first);
    }
    writer.json.raw("]");
}

fn is_warning(error: &SurfaceError) -> bool {
    error.is_recoverable() || error.code.is_recovery()
}

fn surface(writer: &mut Writer<'_>, error: &SurfaceError, first: &mut bool) {
    separator(writer, first);
    let span = error.span(writer.source.len() as u32);
    writer.json.raw("{\"code\":");
    writer.json.string(error.code.code());
    writer.json.raw(",\"message\":");
    writer.json.string(&error.message(writer.source));
    writer.json.raw(",\"loc\":");
    writer.loc(span.start, span.end);
    writer.json.raw("}");
}

fn separator(writer: &mut Writer<'_>, first: &mut bool) {
    if !*first {
        writer.json.raw(",");
    }
    *first = false;
}
