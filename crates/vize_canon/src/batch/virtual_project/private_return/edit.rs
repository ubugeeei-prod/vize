//! Own the proposed bytes/maps until the final import rewrite is sealed.

use std::ops::Range;

use vize_carton::String;

use crate::{
    batch::RewriteResult,
    virtual_ts::{ProjectionMapping, VizeSemanticLink},
};

use super::syntax::ReturnSite;

pub(super) struct ProposedEdit {
    pub(super) code: String,
    pub(super) projection: ProjectionMapping,
    replacement: String,
    range: Range<usize>,
}

pub(super) fn propose(
    source: &str,
    projection: &ProjectionMapping,
    site: &ReturnSite,
    relative_module: &str,
) -> Option<ProposedEdit> {
    if !relative_module.is_ascii()
        || !(relative_module.starts_with("./") || relative_module.starts_with("../"))
        || relative_module.contains('\\')
        || !relative_module.ends_with(".js")
        || source.get(site.range.clone()).is_none()
        || projection.spans().iter().any(|row| {
            overlaps(&row.gen_range, &site.range)
                || row
                    .sub_spans
                    .iter()
                    .any(|sub| overlaps(&sub.gen_range, &site.range))
        })
        || projection
            .semantic_links()
            .iter()
            .chain(projection.prop_default_key_links())
            .any(|link| link_overlaps(link, &site.range))
    {
        return None;
    }
    let module = serde_json::to_string(relative_module).ok()?;
    let replacement = vize_carton::cstr!(
        "import({module}).default<{}, {}, $AuthoredProps, $ComponentInstance>",
        site.props,
        site.declared
    );
    let mut code = String::from(source);
    code.replace_range(site.range.clone(), &replacement);
    let mut projection = projection.clone();
    projection.note_generated_replacement(site.range.start, site.range.len(), replacement.len());
    Some(ProposedEdit {
        code,
        projection,
        replacement,
        range: site.range.clone(),
    })
}

impl ProposedEdit {
    /// The normal rewriter resolves relative declaration imports against the
    /// ORIGINAL SFC directory. A previously unreferenced original `.d.ts` can
    /// therefore intercept this new spelling. Seal the WHOLE introduced
    /// expression after that real pass, using its actual offset map. This is
    /// a refusal on an unexpected rewrite, never path/diagnostic normalization.
    pub(super) fn accepts_rewrite(&self, rewritten: &RewriteResult) -> bool {
        let Ok(start) = u32::try_from(self.range.start) else {
            return false;
        };
        let Some(end) = self.range.start.checked_add(self.replacement.len()) else {
            return false;
        };
        let Ok(end) = u32::try_from(end) else {
            return false;
        };
        let generated_start = rewritten.source_map.get_virtual_offset(start) as usize;
        let generated_end = rewritten.source_map.get_virtual_offset(end) as usize;
        generated_end.checked_sub(generated_start) == Some(self.replacement.len())
            && rewritten.code.get(generated_start..generated_end) == Some(self.replacement.as_str())
            && rewritten
                .source_map
                .get_original_offset(generated_start as u32)
                == start
            && rewritten
                .source_map
                .get_original_offset(generated_end as u32)
                == end
    }
}

fn overlaps(range: &Range<usize>, edit: &Range<usize>) -> bool {
    if range.is_empty() {
        edit.start <= range.start && range.start < edit.end
    } else {
        range.start < edit.end && edit.start < range.end
    }
}

fn link_overlaps(link: &VizeSemanticLink, edit: &Range<usize>) -> bool {
    overlaps(&link.source_range, edit) || overlaps(&link.target_range, edit)
}

#[cfg(test)]
mod tests;
