//! Project only imports whose existing atomic range proves generated ownership.

use vize_carton::{String, cstr};

use super::{ImportSourceMap, RewriteResult};
use crate::batch::virtual_specifier_message::MISSING_VUE_IMPORT_SENTINEL;

impl ImportSourceMap {
    /// Keep unrelated bytes and all original coordinates exactly as registered.
    /// The callback rechecks graph ownership on each finalization; no resolution
    /// decision survives a target removal, replacement or project change.
    pub(crate) fn project_owned_relative_vue_imports(
        &self,
        pristine: &str,
        current: &str,
        project_specifier: impl Fn(&str) -> Option<String>,
    ) -> Option<RewriteResult> {
        if !self.adjustments.is_empty() {
            return None;
        }
        let mut output = None;
        let mut lengths = Vec::new();
        // Existing generated ranges, not a textual search or a second AST parse.
        for (index, range) in self.replacement_ranges.iter().enumerate().rev() {
            let Some(original) =
                pristine.get(range.original_start as usize..range.original_end as usize)
            else {
                continue;
            };
            if !(original.starts_with("./") || original.starts_with("../"))
                || !original.ends_with(".vue")
                || original
                    .bytes()
                    .any(|byte| matches!(byte, b'\\' | b'\'' | b'"') || byte < 0x20)
                || original.contains(['\u{2028}', '\u{2029}'])
            {
                continue;
            }
            let Some(existing) =
                current.get(range.virtual_start as usize..range.virtual_end as usize)
            else {
                continue;
            };
            let old = cstr!("{original}.ts");
            let proposed = cstr!("{original}.js");
            let missing = cstr!("{old}{MISSING_VUE_IMPORT_SENTINEL}");
            if existing != old.as_str()
                && existing != proposed.as_str()
                && existing != missing.as_str()
            {
                continue;
            }
            let Some(replacement) = project_specifier(original) else {
                continue;
            };
            if replacement.as_str() == existing {
                continue;
            }
            let code = output.get_or_insert_with(|| String::from(current));
            if let Some(length) = super::super::literal_rewrite::replace(
                code,
                current,
                range.virtual_start,
                range.virtual_end,
                &replacement,
            ) {
                lengths.push((index, length));
            }
        }
        let code = output?;
        if lengths.is_empty() {
            return None;
        }
        // Rebuild the same original intervals. Only an admitted literal's
        // emitted length can move later generated coordinates.
        let mut source_map = Self::for_rewrites(self.replacement_ranges.len());
        let mut changed = lengths.into_iter().peekable();
        for (index, range) in self.replacement_ranges.iter().enumerate().rev() {
            let length = if changed.peek().is_some_and(|(changed, _)| *changed == index) {
                changed.next().unwrap().1
            } else {
                range.virtual_end - range.virtual_start
            };
            source_map.record_replacement(range.original_start, range.original_end, length);
        }
        source_map.finish_descending_replacements();
        Some(RewriteResult { code, source_map })
    }
}
