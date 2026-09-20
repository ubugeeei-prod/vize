use std::ops::Range;
use vize_armature::patterns::attribute_source_offset;
use vize_carton::{String, cstr};

use super::PatternContext;
use crate::virtual_ts::expressions::{
    map_rewritten_template_binding, rewrite_reserved_template_binding,
};
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};

pub(super) struct PatternEmitter<'a, 'b> {
    pub ctx: &'a PatternContext<'b>,
    pub indent: &'a str,
    prefix: String,
    next: u32,
}

impl<'a, 'b> PatternEmitter<'a, 'b> {
    pub fn new(ctx: &'a PatternContext<'b>, prefix: String, indent: &'a str) -> Self {
        Self {
            ctx,
            indent,
            prefix,
            next: 0,
        }
    }

    pub fn name(&mut self) -> String {
        let name = cstr!("{}{}", self.prefix, self.next);
        self.next += 1;
        name
    }

    pub fn expression(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        text: &str,
        start: u32,
        end: u32,
    ) -> String {
        let expression = rewrite_reserved_template_binding(text, self.ctx.template_binding_access)
            .unwrap_or_else(|| String::from(text));
        let gen_start = ts.len();
        ts.push_str(&expression);
        if self.ctx.verification {
            let source_start = (self.ctx.template_offset + start) as usize;
            let mut mapping = VizeMapping {
                gen_range: gen_start..ts.len(),
                src_range: source_start..(self.ctx.template_offset + end) as usize,
                sub_spans: Vec::new(),
            };
            if expression == text
                && let Some(raw) = self
                    .ctx
                    .template_source
                    .and_then(|source| source.get(start as usize..end as usize))
            {
                // Entity expansion makes a linear byte map incorrect. Keep
                // identifier spans only for expressions that need them.
                if raw != text {
                    let mut chars = text.char_indices().peekable();
                    while let Some((at, ch)) = chars.next() {
                        let mut end = at + ch.len_utf8();
                        if oxc_syntax::identifier::is_identifier_part(ch) {
                            while let Some(&(next, ch)) = chars.peek() {
                                if !oxc_syntax::identifier::is_identifier_part(ch) {
                                    break;
                                }
                                end = next + ch.len_utf8();
                                chars.next();
                            }
                        }
                        mapping.sub_spans.push(VizeSubSpan {
                            gen_range: gen_start + at..gen_start + end,
                            src_range: source_start
                                + attribute_source_offset(raw, at as u32) as usize
                                ..source_start + attribute_source_offset(raw, end as u32) as usize,
                        });
                    }
                }
            }
            mappings.push(mapping);
            map_rewritten_template_binding(
                ts,
                mappings,
                gen_start,
                source_start,
                text,
                self.ctx.template_binding_access,
            );
        }
        expression
    }
}

pub(super) fn map_range(
    mappings: &mut Vec<VizeMapping>,
    gen_range: Range<usize>,
    src_range: Range<usize>,
) {
    mappings.push(VizeMapping {
        gen_range,
        src_range,
        sub_spans: Vec::new(),
    });
}
