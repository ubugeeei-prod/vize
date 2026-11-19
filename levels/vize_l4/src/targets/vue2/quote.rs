use crate::write::{LinkSink, Writer};
use vize_l0::{Span, ToCompactString};

pub(super) fn quoted<L: LinkSink>(writer: &mut Writer<L>, value: &str, span: Span) {
    let quoted = serde_json::Value::from(value).to_compact_string();
    writer.push_linked(&quoted, span);
}

pub(super) fn property<L: LinkSink>(writer: &mut Writer<L>, value: &str, span: Span) {
    // A literal __proto__ key is a setter even when quoted. Native attributes
    // are own data properties, so encode this spelling as a computed key.
    if value == "__proto__" {
        writer.push("[");
    }
    quoted(writer, value, span);
    if value == "__proto__" {
        writer.push("]");
    }
}
