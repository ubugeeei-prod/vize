//! Evaluate captured type expressions at their authored control-flow position.

use oxc_span::Span;
use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use crate::virtual_ts::VizeMapping;

mod emit;

#[derive(Default)]
pub(in crate::virtual_ts::generator) struct AmbientProjection {
    captures: Vec<Capture>,
    prefix: String,
    diagnostic_mappings: Vec<VizeMapping>,
}

struct Capture {
    owner: Span,
    expression: Span,
    value: bool,
}

impl AmbientProjection {
    pub(in crate::virtual_ts::generator) fn plan(
        summary: &Croquis,
        script: Option<&str>,
        module_spans: &mut Vec<(u32, u32)>,
    ) -> Self {
        super::extend_module_spans(summary, script, module_spans)
    }

    pub(super) fn collect<'a>(
        &mut self,
        candidates: &[Span],
        captured: Vec<Vec<(Span, bool)>>,
        blocked: &[bool],
        names: impl Iterator<Item = &'a str>,
    ) {
        for ((owner, expressions), blocked) in candidates.iter().zip(captured).zip(blocked) {
            if !blocked {
                self.captures
                    .extend(expressions.into_iter().map(|(expression, value)| Capture {
                        owner: *owner,
                        expression,
                        value,
                    }));
            }
        }
        self.captures.sort_by_key(|capture| {
            (
                capture.expression.start,
                std::cmp::Reverse(capture.expression.end),
            )
        });
        let mut end = 0;
        self.captures.retain(|capture| {
            if capture.expression.start < end {
                return false;
            }
            end = capture.expression.end;
            true
        });
        if self.captures.is_empty() {
            return;
        }
        let used: FxHashSet<usize> = names
            .filter_map(|name| {
                name.strip_prefix("__vize_ambient_")?
                    .split_once('_')?
                    .0
                    .parse()
                    .ok()
            })
            .collect();
        let mut id = 0;
        while used.contains(&id) {
            id += 1;
        }
        self.prefix = cstr!("__vize_ambient_{id}_");
    }

    pub(in crate::virtual_ts::generator) fn script_lines<'a>(
        &'a self,
        script: &'a str,
    ) -> impl Iterator<Item = (usize, &'a str)> + 'a {
        let mut offset = 0;
        script.split('\n').flat_map(move |line| {
            let start = offset;
            offset += line.len() + 1;
            let end = start + line.len();
            let first = self
                .captures
                .partition_point(|capture| capture.owner.start as usize <= start);
            let mut splits = self
                .captures
                .get(first..)
                .unwrap_or_default()
                .iter()
                .take_while(move |capture| (capture.owner.start as usize) < end)
                .map(|capture| capture.owner.start as usize)
                .peekable();
            let mut cursor = start;
            let mut done = false;
            std::iter::from_fn(move || {
                if done {
                    return None;
                }
                let next = splits.next().unwrap_or_else(|| {
                    done = true;
                    end
                });
                while splits.peek() == Some(&next) {
                    splits.next();
                }
                let chunk = (
                    cursor,
                    line.get(cursor - start..next - start).unwrap_or_default(),
                );
                cursor = next;
                Some(chunk)
            })
        })
    }

    pub(in crate::virtual_ts::generator) fn emit_setup_captures(
        &self,
        start: usize,
        script: &str,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        source_offset: impl Fn(usize) -> usize,
    ) {
        let first = self
            .captures
            .partition_point(|capture| (capture.owner.start as usize) < start);
        for (index, capture) in self
            .captures
            .iter()
            .enumerate()
            .skip(first)
            .take_while(|(_, capture)| capture.owner.start as usize == start)
        {
            let prefix = &self.prefix;
            if capture.value {
                append!(*ts, "  const {prefix}value_{index} = ");
            } else {
                append!(*ts, "  type {prefix}type_{index} = ");
            }
            emit::mapped(ts, mappings, script, capture.expression, &source_offset);
            ts.push_str(";\n");
            if capture.value {
                append!(
                    *ts,
                    "  type {prefix}type_{index} = typeof {prefix}value_{index};\n"
                );
            }
        }
    }

    pub(in crate::virtual_ts::generator) fn emit_return(
        &self,
        ts: &mut String,
        fields: &[String],
        mappings: &mut Vec<VizeMapping>,
    ) {
        if self.captures.is_empty() {
            if !fields.is_empty() {
                append!(*ts, "\n  return {{ {} }};\n", fields.join(", "));
            }
            return;
        }
        // Explicit, lazy accessor types avoid both return-object widening and
        // synthetic cycles through exposed values or exported type artifacts.
        ts.push_str("\n  return {\n");
        for field in fields {
            append!(*ts, "    get {field}() {{ return {field}; }},\n");
        }
        for index in 0..self.captures.len() {
            let prefix = &self.prefix;
            append!(
                *ts,
                "    get {prefix}capture_{index}(): {prefix}type_{index} {{ return null!; }},\n"
            );
        }
        ts.push_str("  };\n");
        // Authored expression mappings come first for editor requests; the
        // replacement constructor name still needs an authored diagnostic target.
        mappings.extend(self.diagnostic_mappings.iter().cloned());
    }
}
