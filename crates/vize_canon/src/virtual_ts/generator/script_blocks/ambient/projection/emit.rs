use oxc_span::Span;
use vize_carton::{String, append};

use super::AmbientProjection;
use crate::virtual_ts::VizeMapping;

impl AmbientProjection {
    pub(in crate::virtual_ts::generator) fn emit_module_statement(
        &mut self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        script: &str,
        span: (u32, u32),
        source_offset: impl Fn(usize) -> usize,
    ) -> bool {
        let first = self
            .captures
            .partition_point(|capture| capture.expression.start < span.0);
        let captures = self.captures[first..]
            .iter()
            .enumerate()
            .take_while(|(_, capture)| capture.expression.end <= span.1);
        if self
            .captures
            .get(first)
            .is_none_or(|capture| capture.expression.end > span.1)
        {
            return false;
        }
        for (relative, capture) in captures.clone() {
            if capture.value {
                let index = first + relative;
                let prefix = &self.prefix;
                append!(
                    *ts,
                    "declare const {prefix}base_{index}: Awaited<ReturnType<typeof __setup>>[\"{prefix}capture_{index}\"];\n"
                );
            }
        }
        let mut start = span.0;
        for (relative, capture) in captures {
            let index = first + relative;
            mapped(
                ts,
                mappings,
                script,
                Span::new(start, capture.expression.start),
                &source_offset,
            );
            let prefix = &self.prefix;
            if capture.value {
                let start = ts.len();
                append!(*ts, "{prefix}base_{index}");
                self.diagnostic_mappings.push(VizeMapping {
                    gen_range: start..ts.len(),
                    src_range: source_offset(capture.expression.start as usize)
                        ..source_offset(capture.expression.end as usize),
                    sub_spans: Vec::new(),
                });
            } else {
                append!(
                    *ts,
                    "Awaited<ReturnType<typeof __setup>>[\"{prefix}capture_{index}\"]"
                );
            }
            start = capture.expression.end;
        }
        mapped(
            ts,
            mappings,
            script,
            Span::new(start, span.1),
            &source_offset,
        );
        ts.push('\n');
        true
    }
}

pub(super) fn mapped(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    script: &str,
    span: Span,
    source_offset: &impl Fn(usize) -> usize,
) {
    if span.is_empty() {
        return;
    }
    let start = ts.len();
    ts.push_str(&script[span.start as usize..span.end as usize]);
    mappings.push(VizeMapping {
        gen_range: start..ts.len(),
        src_range: source_offset(span.start as usize)..source_offset(span.end as usize),
        sub_spans: Vec::new(),
    });
}
