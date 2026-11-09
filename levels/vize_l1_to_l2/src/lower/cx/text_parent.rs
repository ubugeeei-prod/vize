//! Text rules belong to the direct owner; pre whitespace also spans descendants.
use vize_l1::markup::Element;
use vize_l2::provenance::ProvenanceSink;

use super::{Cx, TextWhitespace};

impl<'a, P: ProvenanceSink> Cx<'a, P> {
    pub(crate) fn enter_text_parent(
        &mut self,
        element: &Element<'a>,
        tag: &str,
        html: bool,
    ) -> (TextWhitespace, Option<u32>) {
        let previous = (self.text_whitespace, self.ignore_newline_at);
        // The caller has already pushed this element's own `<pre>` and
        // `v-pre` suppression, so the lane's subtree flags stand.
        let lane = self.text_lane().children_of(element, tag, html, false);
        self.text_whitespace = lane.whitespace;
        self.ignore_newline_at = lane.ignore_newline_at;
        previous
    }
}
