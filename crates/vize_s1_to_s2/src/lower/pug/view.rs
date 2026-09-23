//! A host source (an SFC) with its pug block swapped for the derived Vue
//! template — how tools that read the HTML template lane (Patina today,
//! any `template.content` consumer) run unchanged over pug — plus the exact
//! map from a position in that view back to the authored host source.
//!
//! The view uses [`PugRendering::AuthoredOrder`]: pug hoists the merged
//! `class` attribute to the front, which would make order rules report
//! what pug did rather than what the author wrote.
//!
//! Positions before the block are unchanged, positions after it shift by
//! the length difference, and positions inside it go through the
//! [`PugSourceMap`](super::PugSourceMap): the transform is positional, so
//! no diagnostic can be attributed to the wrong block.

use core::ops::Range;

use vize_s0::String;

use super::{PugRendering, PugTemplate, derive_template_source_with};

/// The host source with one pug block replaced by its derived template.
#[derive(Debug, Clone)]
pub struct PugBlockView {
    /// The view: the host source, block body replaced.
    pub source: String,
    /// The derivation (map and diagnostics) behind the swap.
    pub template: PugTemplate,
    body_start: u32,
    pug_len: u32,
    html_len: u32,
}

impl PugBlockView {
    /// Swap `host[body]` (a pug template body) for its derived template.
    /// A template the lowering refuses has no view: `Err` carries the
    /// derivation with its diagnostics.
    pub fn new(host: &str, body: Range<usize>) -> Result<Self, PugTemplate> {
        let (Some(before), Some(pug), Some(after)) = (
            host.get(..body.start),
            host.get(body.clone()),
            host.get(body.end..),
        ) else {
            return Err(refuse_range());
        };
        let template = derive_template_source_with(pug, PugRendering::AuthoredOrder);
        if template.has_errors() {
            return Err(template);
        }
        let mut source = String::with_capacity(host.len() + template.html.len());
        source.push_str(before);
        source.push_str(&template.html);
        source.push_str(after);
        Ok(Self {
            source,
            body_start: body.start as u32,
            pug_len: (body.end - body.start) as u32,
            html_len: template.html.len() as u32,
            template,
        })
    }

    /// Whether the view range `[start, end)` reaches into the swapped body.
    pub fn touches_body(&self, start: u32, end: u32) -> bool {
        start < self.body_start + self.html_len && end > self.body_start
            || (start == end && start == self.body_start)
    }

    /// The authored host range of the view range `[start, end)`.
    pub fn to_host(&self, start: u32, end: u32) -> (u32, u32) {
        let body_end = self.body_start + self.html_len;
        let point = |offset: u32| {
            if offset <= self.body_start {
                offset
            } else if offset >= body_end {
                offset - self.html_len + self.pug_len
            } else {
                self.body_start + self.template.map.offset_to_pug(offset - self.body_start)
            }
        };
        if start >= self.body_start && start < body_end && end > start && end <= body_end {
            let span = self.template.map.to_pug(vize_s0::Span::new(
                start - self.body_start,
                end - self.body_start,
            ));
            return (self.body_start + span.start, self.body_start + span.end);
        }
        let (from, to) = (point(start), point(end));
        (from, to.max(from))
    }
}

/// The derivation for a `body` range that is not a slice of the host: an
/// empty template carrying one refusal.
fn refuse_range() -> PugTemplate {
    let mut template = derive_template_source_with("", PugRendering::AuthoredOrder);
    template.diagnostics.push(crate::exemptions::lowering(
        vize_s0::Span::new(0, 0),
        "the pug block range is not a UTF-8 slice of the host source",
    ));
    template
}

#[cfg(test)]
mod tests {
    use super::PugBlockView;

    #[test]
    fn view_positions_map_back_to_the_host() {
        let host = "<template lang=\"pug\">\np(title=\"x\") Hi\n</template>\n<script>a</script>";
        let start = host.find('>').unwrap() + 1;
        let end = host.find("</template>").unwrap();
        let view = PugBlockView::new(host, start..end).expect("static pug");
        assert_eq!(
            view.source.as_str(),
            "<template lang=\"pug\"><p title=\"x\">Hi</p></template>\n<script>a</script>"
        );
        let at = |needle: &str, text: &str| text.find(needle).unwrap() as u32;
        // Inside the block: verbatim text maps byte for byte.
        let hi = at("Hi", &view.source);
        assert_eq!(
            view.to_host(hi, hi + 2),
            (at("Hi", host), at("Hi", host) + 2)
        );
        // After the block: a plain shift.
        let script = at("<script>", &view.source);
        let expected = at("<script>", host);
        assert_eq!(view.to_host(script, script + 8), (expected, expected + 8));
        // Before the block: unchanged.
        assert_eq!(view.to_host(1, 9), (1, 9));
        assert!(view.touches_body(hi, hi + 1) && !view.touches_body(script, script + 1));
    }

    #[test]
    fn refused_pug_has_no_view() {
        let host = "- var x = 1";
        let refused = PugBlockView::new(host, 0..host.len()).expect_err("refused");
        assert!(refused.has_errors());
    }
}
