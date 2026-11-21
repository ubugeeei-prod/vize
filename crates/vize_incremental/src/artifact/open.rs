//! Build resident identities directly from one already-parsed descriptor.
use super::{BlockKind, HeaderAttrs, slot};
use crate::{SourceFile, db::Block};
use vize_croquis::sfc::{BlockLocation, SfcDescriptor};
use vize_l0::{FxHashMap, String};

pub(crate) fn blocks<'db, 'descriptor, 'source>(
    db: &'db dyn salsa::Database,
    file: SourceFile,
    text: &str,
    descriptor: &'descriptor SfcDescriptor<'source>,
) -> Vec<Block<'db>> {
    let mut headers = Vec::with_capacity(
        usize::from(descriptor.template.is_some())
            + usize::from(descriptor.script.is_some())
            + usize::from(descriptor.script_setup.is_some())
            + descriptor.styles.len()
            + descriptor.custom_blocks.len(),
    );
    let mut push =
        |kind, ordinal, loc: &BlockLocation, attrs: &'descriptor HeaderAttrs<'source>| {
            headers.push((loc.start, loc.end, kind, ordinal, attrs));
        };
    if let Some(template) = &descriptor.template {
        push(BlockKind::Template, 0, &template.loc, &template.attrs);
    }
    for (kind, script) in [
        (BlockKind::Script, &descriptor.script),
        (BlockKind::ScriptSetup, &descriptor.script_setup),
    ] {
        if let Some(script) = script {
            push(kind, 0, &script.loc, &script.attrs);
        }
    }
    for (ordinal, style) in descriptor.styles.iter().enumerate() {
        push(BlockKind::Style, ordinal, &style.loc, &style.attrs);
    }
    let mut ordinals = FxHashMap::default();
    for custom in &descriptor.custom_blocks {
        let tag = custom.block_type.as_ref();
        let ordinal = ordinals.entry(tag).or_insert(0);
        push(
            BlockKind::Custom(String::from(tag)),
            *ordinal,
            &custom.loc,
            &custom.attrs,
        );
        *ordinal += 1;
    }
    // Order lightweight headers before constructing the original block source
    // and its tracked identity. No intermediate vector of owned slots is kept.
    headers.sort_by_key(|header| header.0);
    headers
        .into_iter()
        .filter_map(|(start, end, kind, ordinal, attrs)| {
            let owned = slot(text, kind, ordinal, start, end, attrs)?;
            Some(Block::new(
                db,
                file,
                owned.kind,
                owned.ordinal,
                owned.source,
                owned.start,
            ))
        })
        .collect()
}
