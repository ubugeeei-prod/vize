//! Block stages from the descriptor already owned by a named open document.
use super::DocumentBlocks;
use crate::{
    DescriptorParseError, ParsedSfc, ResidentDatabase, SharedDescriptor, SourceFile, db::Block,
    descriptor::sfc_descriptor,
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
    crate::artifact::open::blocks(db, file, file.text(db), descriptor)
}

pub(crate) fn read<'a>(
    db: &'a ResidentDatabase,
    file: SourceFile,
    text: &'a str,
) -> Result<ParsedBlocks<'a>, &'a DescriptorParseError> {
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
            source: text,
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
