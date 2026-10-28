//! Exact script spans around inferred template-ref calls.

use super::TemplateRefKeyChecks;
use crate::virtual_ts::{VizeMapping, VizeSubSpan};
use std::borrow::Cow;
use vize_carton::{String, append};

impl TemplateRefKeyChecks<'_> {
    /// Append one authored script line with exact spans for specialized callees.
    pub(in crate::virtual_ts::generator) fn emit_line(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        mut line: Cow<'_, str>,
        (start, original_length): (usize, usize),
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        let gen_start = ts.len();
        let sub_spans = self.specialize_line(&mut line, start, gen_start, source_offset);
        ts.push_str(&line);
        let gen_end = ts.len();
        ts.push('\n');
        if original_length != 0 {
            let src_start = source_offset(start);
            mappings.push(VizeMapping {
                gen_range: gen_start..gen_end,
                src_range: src_start..src_start + original_length,
                sub_spans,
            });
        }
    }

    /// Preserve exact authored offsets around aliases of any length, including
    /// calls whose arguments continue onto the next line.
    pub(in crate::virtual_ts::generator) fn specialize_line(
        &self,
        line: &mut Cow<'_, str>,
        start: usize,
        gen_start: usize,
        source_offset: &dyn Fn(usize) -> usize,
    ) -> Vec<VizeSubSpan> {
        let end = start + line.len();
        let first = self.calls.partition_point(|call| call.callee.start < start);
        let remaining = self.calls.get(first..).unwrap_or_default();
        let last = remaining.partition_point(|call| call.callee.end <= end);
        let calls = remaining.get(..last).unwrap_or_default();
        if calls.is_empty() {
            return Vec::new();
        }
        if calls
            .iter()
            .all(|call| call.imported && call.callee.len() == self.helper.len())
        {
            for call in calls {
                let range = call.callee.start - start..call.callee.end - start;
                if line.get(range.clone()) == Some(call.name.as_str()) {
                    line.to_mut().replace_range(range, self.helper);
                }
            }
            return Vec::new();
        }
        let selected = calls.iter().filter(|call| {
            line.get(call.callee.start - start..call.callee.end - start) == Some(call.name.as_str())
        });
        if selected.clone().next().is_none() {
            return Vec::new();
        }
        let mut output = String::with_capacity(line.len() + calls.len() * self.helper.len());
        let mut spans = Vec::with_capacity(calls.len() * 2 + 1);
        let mut cursor = 0;
        for call in selected {
            let range = call.callee.start - start..call.callee.end - start;
            if let Some(prefix) = line.get(cursor..range.start)
                && !prefix.is_empty()
            {
                spans.push(VizeSubSpan {
                    gen_range: gen_start + output.len()..gen_start + output.len() + prefix.len(),
                    src_range: source_offset(start + cursor)..source_offset(start + range.start),
                });
                output.push_str(prefix);
            }
            if !call.imported {
                output.push('(');
            }
            let callee = if call.imported {
                self.helper
            } else {
                call.name.as_str()
            };
            spans.push(VizeSubSpan {
                gen_range: gen_start + output.len()..gen_start + output.len() + callee.len(),
                src_range: source_offset(call.callee.start)..source_offset(call.callee.end),
            });
            output.push_str(callee);
            if !call.imported {
                append!(output, " as {}Global)", self.helper);
            }
            cursor = range.end;
        }
        if let Some(tail) = line.get(cursor..)
            && !tail.is_empty()
        {
            spans.push(VizeSubSpan {
                gen_range: gen_start + output.len()..gen_start + output.len() + tail.len(),
                src_range: source_offset(start + cursor)..source_offset(end),
            });
            output.push_str(tail);
        }
        *line = Cow::Owned(output.into());
        spans
    }
}
