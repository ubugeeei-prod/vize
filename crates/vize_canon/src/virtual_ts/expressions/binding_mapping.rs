//! Compose cached relative binding spans with one actual emitted value.

use std::ops::Range;

use super::spread_reserved_props::BindingRewrite;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::VizeMapping;

pub(super) struct BindingSource<'a> {
    pub(super) expression: &'a str,
    pub(super) start: usize,
    /// Authored byte for each expression byte; synthetic comment spaces have none.
    pub(super) offsets: Option<&'a [u32]>,
}

impl BindingSource<'_> {
    fn anchor(&self, relative: Range<usize>) -> Option<Range<usize>> {
        let Some(offsets) = self.offsets else {
            return Some(self.start + relative.start..self.start + relative.end);
        };
        let bytes = offsets.get(relative)?;
        let first = *bytes.first()?;
        if first == u32::MAX
            || !bytes
                .iter()
                .enumerate()
                .all(|(index, offset)| *offset == first + index as u32)
        {
            return None;
        }
        Some(self.start + first as usize..self.start + first as usize + bytes.len())
    }

    fn append_navigation(
        &self,
        mappings: &mut Vec<VizeMapping>,
        generated: Range<usize>,
        relative: Range<usize>,
    ) {
        let Some(offsets) = self.offsets else {
            mappings.push(VizeMapping {
                gen_range: generated,
                src_range: self.start + relative.start..self.start + relative.end,
                sub_spans: Vec::new(),
            });
            return;
        };
        let Some(bytes) = offsets.get(relative) else {
            return;
        };
        let mut cursor = 0;
        while let Some(&first) = bytes.get(cursor) {
            if first == u32::MAX {
                cursor += 1;
                continue;
            }
            let run_start = cursor;
            cursor += 1;
            while bytes
                .get(cursor)
                .is_some_and(|offset| *offset == first + (cursor - run_start) as u32)
            {
                cursor += 1;
            }
            mappings.push(VizeMapping {
                gen_range: generated.start + run_start..generated.start + cursor,
                src_range: self.start + first as usize
                    ..self.start + first as usize + cursor - run_start,
                sub_spans: Vec::new(),
            });
        }
    }
}

pub(super) fn map_rewrite(
    emitted: &str,
    mappings: &mut Vec<VizeMapping>,
    generated: Range<usize>,
    source: &BindingSource<'_>,
    rewrite: &BindingRewrite,
    bindings: &TemplateBindingAccess,
) {
    for (relative, _) in emitted
        .get(generated.clone())
        .unwrap_or_default()
        .match_indices(rewrite.code.as_str())
    {
        let base = generated.start + relative;
        for span in &rewrite.diagnostic_prefixes {
            if let Some(authored) = source.anchor(span.source.clone()) {
                bindings.record_instance_diagnostic_prefix(VizeMapping {
                    gen_range: base + span.generated.start..base + span.generated.end,
                    src_range: authored,
                    sub_spans: Vec::new(),
                });
            }
        }
        for span in &rewrite.segments {
            source.append_navigation(
                mappings,
                base + span.generated.start..base + span.generated.end,
                span.source.clone(),
            );
        }
    }
}
