//! AST-owned default exports whose keywords span comments or physical lines.

use super::{options_api::OptionsApiScriptFacts, template_ref_keys::TemplateRefKeyChecks};
use crate::virtual_ts::{TextBuf, VizeMapping};
use std::ops::Range;
use vize_carton::String;

/// Locate `default` after an AST-proven `export`, preserving its trivia.
pub(super) fn keyword_end(source: &str, start: usize) -> Option<usize> {
    source.get(start..)?.strip_prefix("export")?;
    let mut at = start + 6;
    loop {
        let rest = source.get(at..)?;
        let trimmed = rest.trim_start_matches(|c: char| {
            c == '\u{feff}' || (c.is_whitespace() && c != '\u{0085}')
        });
        at += rest.len() - trimmed.len();
        if trimmed.starts_with("//") {
            at += trimmed.find(['\r', '\n', '\u{2028}', '\u{2029}'])?;
        } else if trimmed.starts_with("/*") {
            at += trimmed.find("*/")? + 2;
        } else {
            return trimmed.strip_prefix("default").map(|_| at + 7);
        }
    }
}

pub(super) struct IrregularDefault<'a> {
    export: Range<usize>,
    default: Range<usize>,
    close: Option<usize>,
    class: Option<(usize, &'a str)>,
}

pub(super) struct ProjectedLine {
    pub text: String,
    pub declares_alias: bool,
    copied: Vec<(Range<usize>, Range<usize>)>,
}

impl<'a> IrregularDefault<'a> {
    pub(super) fn new(script: &'a str, facts: &OptionsApiScriptFacts) -> Option<Self> {
        let targets = &facts.default_export;
        let (start, inner, close, class) = if let Some((start, inner, end)) = targets.object {
            (start, inner, Some(end), None)
        } else if let Some((start, inner, end, ns, ne)) = targets.class {
            (start, inner, None, Some((end, script.get(ns..ne)?)))
        } else {
            let (start, inner, _) = targets.expr?;
            (start, inner, None, None)
        };
        let before = script.get(..start)?;
        let line_start = before.rfind('\n').map_or(0, |line| line + 1);
        if script.get(start..inner)?.starts_with("export default")
            && before.get(line_start..)?.trim().is_empty()
        {
            return None;
        }
        let end = keyword_end(script, start)?;
        Some(Self {
            export: start..start + 6,
            default: end - 7..end,
            close,
            class,
        })
    }

    pub(super) fn project_line(
        &self,
        line: &str,
        start: usize,
        const_assertions: &[usize],
    ) -> Option<ProjectedLine> {
        let end = start + line.len();
        let declares_alias = start <= self.export.start && self.export.start < end;
        let mut edits = Vec::new();
        for &offset in const_assertions {
            if start < offset && offset <= end {
                edits.push((offset - start..offset - start, " as const"));
            }
        }
        for (keyword, mask) in [(&self.export, "      "), (&self.default, "       ")] {
            if start <= keyword.start && keyword.end <= end {
                edits.push((keyword.start - start..keyword.end - start, mask));
            }
        }
        if declares_alias {
            let prefix = if self.close.is_some() {
                "const __default__ = __vizeDefineComponent("
            } else if self.class.is_some() {
                ""
            } else {
                "const __default__ ="
            };
            let at = self.export.start - start;
            edits.push((at..at, prefix));
        }
        if let Some(close) = self.close.filter(|&close| start < close && close <= end) {
            edits.push((close - start..close - start, ")"));
        }
        if edits.is_empty() {
            return None;
        }
        edits.sort_by_key(|(range, _)| (range.start, range.end));
        let mut output = ProjectedLine {
            text: String::default(),
            declares_alias,
            copied: Vec::new(),
        };
        let mut cursor = 0;
        for (range, replacement) in edits {
            output.copy(line, cursor..range.start)?;
            output.text.push_str(replacement);
            cursor = range.end;
        }
        output.copy(line, cursor..line.len())?;
        Some(output)
    }

    pub(super) fn class_alias(&self, start: usize, end: usize) -> Option<&'a str> {
        self.class
            .filter(|&(close, _)| start < close && close <= end)
            .map(|(_, name)| name)
    }
}

impl ProjectedLine {
    fn copy(&mut self, line: &str, range: Range<usize>) -> Option<()> {
        if !range.is_empty() {
            let generated = self.text.len()..self.text.len() + range.len();
            self.text.push_str(line.get(range.clone())?);
            self.copied.push((generated, range));
        }
        Some(())
    }

    /// Project each copied source range before composing it with synthetic
    /// export text. Callee aliases and const assertions can both change length.
    pub(super) fn emit(
        &self,
        ts: &mut TextBuf,
        mappings: &mut Vec<VizeMapping>,
        refs: &TemplateRefKeyChecks<'_>,
        start: usize,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        let mut cursor = 0;
        for (generated, authored) in &self.copied {
            ts.push_str(self.text.get(cursor..generated.start).unwrap_or_default());
            if let Some(text) = self.text.get(generated.clone()) {
                refs.emit_mapped_chunk(ts, mappings, text, start + authored.start, source_offset);
            }
            cursor = generated.end;
        }
        ts.push_str(self.text.get(cursor..).unwrap_or_default());
        ts.push('\n');
    }
}

#[cfg(test)]
mod tests;
