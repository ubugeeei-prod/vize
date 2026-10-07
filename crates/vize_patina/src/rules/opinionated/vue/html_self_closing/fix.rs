//! Retain attributes and prove closing delimiters before changing tag shape.

use super::HtmlSelfClosingStyle;
use crate::diagnostic::{Fix, TextEdit};
use crate::rules::opinionated::vue::authored_element::{closing, opening};
use vize_l0::{String, cstr, is_void_tag};
use vize_relief::{ElementNode, Namespace};

pub(super) fn element_fix(
    source: &str,
    element: &ElementNode<'_>,
    style: HtmlSelfClosingStyle,
    help: &str,
) -> Option<Fix> {
    let authored = opening(source, element)?;
    let is_void = element.ns == Namespace::Html && is_void_tag(element.tag);
    let start = element.loc.span.start;
    let mut end = element.loc.span.end;
    let replacement: String = match style {
        HtmlSelfClosingStyle::Always => {
            if !is_void {
                let close = closing(source, element)?;
                // Parsed whitespace may be dropped; no authored content may be lost.
                if !source.get(end as usize..close.start)?.trim().is_empty() {
                    return None;
                }
                end = u32::try_from(close.end).ok()?;
            }
            let prefix = authored.strip_suffix('>')?;
            let separator = if prefix.ends_with([' ', '\t', '\r', '\n', '\u{c}']) {
                ""
            } else {
                " "
            };
            cstr!("{prefix}{separator}/>")
        }
        HtmlSelfClosingStyle::Never => {
            let before_end = authored
                .strip_suffix('>')?
                .trim_end_matches([' ', '\t', '\r', '\n', '\u{c}']);
            let prefix = before_end
                .strip_suffix('/')?
                .trim_end_matches([' ', '\t', '\r', '\n', '\u{c}']);
            if is_void {
                cstr!("{prefix}>")
            } else {
                cstr!("{prefix}></{}>", element.tag)
            }
        }
        HtmlSelfClosingStyle::Any => return None,
    };
    Some(Fix::new(help, TextEdit::replace(start, end, replacement)))
}
