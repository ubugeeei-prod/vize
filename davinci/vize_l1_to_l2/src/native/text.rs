use super::{Context, NativeHoleKind};
use vize_l0::String;
use vize_l1::Token;
use vize_l1::markup::entity::{EntityContext, decode_one, needs_decoding};
use vize_l2::artifact::RegionBuilder;

impl<'a> Context<'a> {
    pub(super) fn text(&mut self, region: &mut RegionBuilder<'_, 'a>, token: &Token<'a>) {
        let span = self.token_span(token);
        let text = decode_text(self.allocator, token.text);
        let result = region.text(text, span);
        self.produced(region, result, "native.text", span, "ui.text");
    }

    pub(super) fn comment(&mut self, region: &mut RegionBuilder<'_, 'a>, token: &Token<'a>) {
        let span = self.token_span(token);
        let Some(body) = token
            .text
            .strip_prefix("<!--")
            .and_then(|body| body.strip_suffix("-->"))
        else {
            self.hole(region, NativeHoleKind::UnexpectedMarkup, span);
            return;
        };
        let result = region.comment(body, span);
        self.produced(region, result, "native.comment", span, "ui.comment");
    }
}

fn decode_text<'a>(allocator: &'a vize_l0::Allocator, raw: &'a str) -> &'a str {
    if !needs_decoding(raw.as_bytes()) {
        return raw;
    }
    let mut output = String::default();
    let mut cursor = 0;
    for (at, _) in raw.match_indices('&') {
        if at < cursor {
            continue;
        }
        let Some((value, consumed)) = raw
            .as_bytes()
            .get(at..)
            .and_then(|bytes| decode_one(bytes, EntityContext::Text))
        else {
            continue;
        };
        let Some(end) = at.checked_add(consumed) else {
            continue;
        };
        if consumed == 0 || raw.get(at..end).is_none() {
            continue;
        }
        if let Some(literal) = raw.get(cursor..at) {
            output.push_str(literal);
        }
        value.for_each(|scalar| output.push(scalar));
        cursor = end;
    }
    if cursor == 0 {
        return raw;
    }
    if let Some(tail) = raw.get(cursor..) {
        output.push_str(tail);
    }
    allocator.alloc_str(output.as_str())
}
