//! Block stages from the descriptor already owned by a named open document.
use super::DocumentBlocks;
use crate::{
    DescriptorParseError, ParsedSfc, ResidentDatabase, SharedDescriptor, SourceFile,
    artifact::descriptor_slots, db::Block, descriptor::sfc_descriptor,
};

/// The original shared descriptor and its block partition at one revision.
pub struct ParsedBlocks<'db> {
    pub descriptor: &'db SharedDescriptor,
    pub blocks: DocumentBlocks<'db>,
}

#[salsa::tracked(returns(ref))]
fn descriptor_blocks(db: &dyn salsa::Database, file: SourceFile) -> Vec<Block<'_>> {
    let ParsedSfc::Descriptor(descriptor) = sfc_descriptor(db, file) else {
        return Vec::new();
    };
    descriptor_slots(file.text(db), descriptor)
        .into_iter()
        .map(|slot| Block::new(db, file, slot.kind, slot.ordinal, slot.source, slot.start))
        .collect()
}

pub(crate) fn read(
    db: &ResidentDatabase,
    file: SourceFile,
) -> Result<ParsedBlocks<'_>, &DescriptorParseError> {
    let descriptor = match sfc_descriptor(db, file) {
        ParsedSfc::Descriptor(descriptor) => descriptor,
        ParsedSfc::Failed(error) => return Err(error),
    };
    let blocks = descriptor_blocks(db, file);
    Ok(ParsedBlocks {
        descriptor,
        blocks: DocumentBlocks {
            db,
            blocks,
            source: file.text(db),
        },
    })
}

pub(crate) fn release(db: &ResidentDatabase, file: SourceFile) {
    // The caller has cleared this source. Revalidating the partition discards
    // its old tracked block identities and their dependent stage artifacts.
    let _released = descriptor_blocks(db, file);
}

#[cfg(all(test, not(feature = "seeded-stale-cache")))]
mod tests;
