//! Emit collected await regions, preserving the established direct-statement maps.
use oxc_span::Span;
use vize_carton::String;

use super::collector::AwaitRegion;
use crate::module_map::Runs;

pub(super) fn rewrite(
    source: &str,
    regions: &[AwaitRegion],
    offset: usize,
    runs: &mut Runs,
) -> Option<String> {
    let mut emitter = Emitter {
        source,
        regions,
        offset,
        index: 0,
        output: String::with_capacity(source.len() + 128),
        runs,
    };
    emitter.range(0, source.len(), true)?;
    (emitter.index == regions.len()).then_some(emitter.output)
}

struct Emitter<'a> {
    source: &'a str,
    regions: &'a [AwaitRegion],
    offset: usize,
    index: usize,
    output: String,
    runs: &'a mut Runs,
}

impl Emitter<'_> {
    fn bounds(&self, span: Span) -> Option<(usize, usize)> {
        let start = (span.start as usize).checked_sub(self.offset)?;
        let end = (span.end as usize).checked_sub(self.offset)?;
        (start <= end && end <= self.source.len()).then_some((start, end))
    }

    fn copy(&mut self, start: usize, end: usize, traced: bool) -> Option<()> {
        let text = self.source.get(start..end)?;
        if traced {
            self.runs.copy(self.output.len(), start, text.len());
        }
        self.output.push_str(text);
        Some(())
    }

    fn range(&mut self, start: usize, end: usize, traced: bool) -> Option<()> {
        let mut cursor = start;
        while let Some(region) = self.regions.get(self.index).copied() {
            let (await_start, await_end) = self.bounds(region.span)?;
            if await_start >= end {
                break;
            }
            if await_start < cursor || await_end > end {
                return None;
            }
            let legacy = match region.legacy_statement {
                Some((span, assignment)) => Some((self.bounds(span)?, assignment)),
                None => None,
            };
            let (replace_start, replace_end) =
                legacy.map_or((await_start, await_end), |(span, _)| span);
            if replace_start < cursor || replace_end > end {
                return None;
            }
            self.copy(cursor, replace_start, traced)?;
            if traced {
                self.runs.point(self.output.len(), replace_start);
            }
            self.copy(replace_start, await_start, false)?;
            self.output
                .push_str(if legacy.is_some_and(|(_, assignment)| assignment) {
                    " (\n"
                } else if region.needs_semicolon {
                    ";(\n"
                } else {
                    "(\n"
                });
            self.output
                .push_str("  ([__temp,__restore] = _withAsyncContext(");
            let await_source = self.source.get(await_start..await_end)?;
            let argument = await_source.strip_prefix("await")?.trim_start();
            if argument.is_empty() {
                return None;
            }
            let argument_start = await_end - argument.len();
            self.index += 1;
            // A setup-owned await in the argument needs an async context callback.
            if self
                .regions
                .get(self.index)
                .is_some_and(|next| next.span.start < region.span.end)
            {
                self.output.push_str("async ");
            }
            self.output.push_str("() => ");
            self.range(argument_start, await_end, traced && legacy.is_none())?;
            self.output.push_str(")),\n  ");
            if !region.is_statement {
                self.output.push_str("__temp = ");
            }
            self.output.push_str("await __temp,\n  __restore()");
            if !region.is_statement {
                self.output.push_str(",\n  __temp");
            }
            self.output.push_str("\n)");
            self.copy(await_end, replace_end, false)?;
            cursor = replace_end;
        }
        self.copy(cursor, end, traced)
    }
}
