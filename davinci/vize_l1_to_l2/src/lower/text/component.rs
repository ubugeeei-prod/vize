//! Vue rendering of a maximal text token from the native component parser.
//! The L1 builder already joins contiguous tokenizer events, including character
//! references. Both neighbours and the byte range still refer to the original
//! child list when its decoded text is projected for the component factory.

use vize_l0::Allocator;
use vize_l1::markup::{Element, SurfaceChild};

use super::condense::{beside_comment, condense_internal, is_vue_ws};
use super::{TextLane, TextSource, TextWhitespace};

impl TextLane {
    pub(crate) fn component_children_of<'a>(
        self,
        src: TextSource<'a>,
        element: &Element<'a>,
        tag: &str,
        html: bool,
        opens_v_pre: bool,
    ) -> Self {
        let mut lane = self.children_of(src, element, tag, html, opens_v_pre);
        // HTML title is RCDATA too. Unlike textarea it keeps its initial LF.
        if html && tag == "title" {
            lane.whitespace = TextWhitespace::Rcdata;
        }
        lane
    }

    pub(crate) fn component_text<'a>(
        self,
        allocator: &'a Allocator,
        decoded: &'a str,
        start: u32,
        previous: Option<&SurfaceChild<'_>>,
        next: Option<&SurfaceChild<'_>>,
    ) -> Option<&'a str> {
        if self.whitespace != TextWhitespace::Normal {
            let text = if self.ignore_newline_at == Some(start) {
                // In preformatted text Vue normalizes CRLF first, so an
                // authored CR followed by CRLF also forms one leading CRLF.
                let normalized_prefix = match self.whitespace {
                    TextWhitespace::Pre => decoded.strip_prefix("\r\r\n"),
                    _ => None,
                };
                normalized_prefix
                    .or_else(|| decoded.strip_prefix("\r\n"))
                    .or_else(|| decoded.strip_prefix('\n'))
                    .unwrap_or(decoded)
            } else {
                decoded
            };
            if self.whitespace == TextWhitespace::Pre && text.contains("\r\n") {
                let mut out = vize_l0::StringBuilder::with_capacity_in(text.len(), allocator);
                let mut parts = text.split("\r\n");
                out.push_str(parts.next().unwrap_or_default());
                for part in parts {
                    out.push('\n');
                    out.push_str(part);
                }
                return Some(out.into_str());
            }
            return Some(text);
        }
        if decoded.chars().all(is_vue_ws) {
            let between_elements = matches!(
                (previous, next),
                (
                    Some(SurfaceChild::Element(_)),
                    Some(SurfaceChild::Element(_))
                )
            );
            if previous.is_none()
                || next.is_none()
                || beside_comment(previous, next)
                || (between_elements && decoded.contains(['\r', '\n']))
            {
                return None;
            }
            return Some(" ");
        }
        Some(condense_internal(allocator, decoded).unwrap_or(decoded))
    }
}
