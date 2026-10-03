//! One append-only HTML-in-JavaScript encoding pass over original L3 parts.

use crate::write::{LinkSink, Writer};
use vize_l3::decision::vapor::VaporPart;

pub(super) fn html_part<L: LinkSink>(writer: &mut Writer<L>, part: &VaporPart<'_, '_>) {
    match *part {
        VaporPart::Open { element, .. } => {
            writer.anchor(element.span.start);
            writer.push("<");
            writer.push(element.tag);
            for attribute in &element.attributes {
                writer.push(" ");
                writer.anchor(attribute.span.start);
                writer.push(attribute.name);
                writer.push("=\\\"");
                html(writer, attribute.value.unwrap_or_default(), true);
                writer.push("\\\"");
            }
            writer.push(">");
        }
        VaporPart::Close { element, .. } => {
            writer.anchor(element.span.end);
            writer.push("</");
            writer.push(element.tag);
            writer.push(">");
        }
        VaporPart::Text { text, .. } => {
            writer.anchor(text.span.start);
            html(writer, text.content, false);
        }
        VaporPart::Comment { comment, .. } => {
            writer.anchor(comment.span.start);
            writer.push("<!--");
            raw(writer, comment.content);
            writer.push("-->");
        }
    }
}

fn html<L: LinkSink>(writer: &mut Writer<L>, text: &str, attribute: bool) {
    for ch in text.chars() {
        match ch {
            '&' => writer.push("&amp;"),
            '<' => writer.push("&lt;"),
            '>' => writer.push("&gt;"),
            '"' if attribute => writer.push("&quot;"),
            _ => scalar(writer, ch),
        }
    }
}

pub(super) fn raw<L: LinkSink>(writer: &mut Writer<L>, text: &str) {
    for ch in text.chars() {
        scalar(writer, ch);
    }
}

fn scalar<L: LinkSink>(writer: &mut Writer<L>, ch: char) {
    match ch {
        '"' => writer.push("\\\""),
        '\\' => writer.push("\\\\"),
        '\n' => writer.push("\\n"),
        '\r' => writer.push("\\r"),
        '\t' => writer.push("\\t"),
        '\u{8}' => writer.push("\\b"),
        '\u{c}' => writer.push("\\f"),
        '\u{2028}' => writer.push("\\u2028"),
        '\u{2029}' => writer.push("\\u2029"),
        _ => writer.push_char(ch),
    }
}
