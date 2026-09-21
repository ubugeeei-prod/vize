//! The in-tree Vue dialect behind the input-dialect contract.
//!
//! Charter #15's first-party tier: the Vue dialect is compiled in behind
//! the same [`InputDialectGuest`] trait an external guest implements, with
//! no transport, and its answer goes through the same acceptance. What it
//! returns is exactly the in-tree boundary: `vize_s1::parse` for the S1
//! page and `vize_s1_to_s2::lower_source_block_with_caps` for the S2 page
//! and the diagnostics — before any S2 pass runs, because the host owns the
//! pass pipeline.

use vize_s0::{Allocator, SourceRoot, String, cstr};
use vize_s1_to_s2::{LegacyCaps, lower_source_block_with_caps};
use vize_s2::folio::S2Folio;

use crate::accept::full_text;
use crate::contract::{
    Capability, Diagnostic, GuestError, InputDialectGuest, LoweredBlock, PROTOCOL_VERSION, Page,
    S1_PAGE_FEATURE, S1_PAGE_SCHEMA, S2_PAGE_FEATURE, S2_PAGE_SCHEMA, SourceBlock,
};
use crate::surface_page::SurfacePage;

/// The `lang` feature the Vue dialect declares.
pub const LANG_HTML: &str = "lang:html";

/// The in-tree Vue template dialect.
#[derive(Debug, Clone, Copy)]
pub struct VueDialect {
    caps: LegacyCaps,
}

impl Default for VueDialect {
    fn default() -> Self {
        Self::new(LegacyCaps::VUE3)
    }
}

impl VueDialect {
    /// The Vue dialect under an explicit capability set (Vue 3 by default).
    #[must_use]
    pub const fn new(caps: LegacyCaps) -> Self {
        Self { caps }
    }
}

/// The Vue dialect's capability offer.
#[must_use]
pub fn capability() -> Capability {
    Capability {
        protocol_version: PROTOCOL_VERSION,
        features: [LANG_HTML, S1_PAGE_FEATURE, S2_PAGE_FEATURE]
            .into_iter()
            .map(String::from)
            .collect(),
    }
}

impl InputDialectGuest for VueDialect {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        Ok(capability())
    }

    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError> {
        lower_block(block, self.caps)
    }
}

/// Lower one block through the in-tree boundary.
///
/// # Errors
///
/// [`GuestError::Trap`] only when `base + source.len()` is not
/// u32-addressable, which no contract block can be.
pub fn lower_block(block: &SourceBlock, caps: LegacyCaps) -> Result<LoweredBlock, GuestError> {
    let base = block.base as usize;
    let fits = u32::try_from(base + block.source.len()).is_ok();
    if !fits {
        return Err(GuestError::Trap(cstr!(
            "block {}+{} is not u32-addressable",
            block.base,
            block.source.len()
        )));
    }
    // S0 spans are file-absolute, and a block frame is a slice of its root:
    // the root is the block behind `base` bytes of ASCII padding, so every
    // span the lowering measures lands at its authored offset.
    let mut root_text = String::with_capacity(base + block.source.len());
    root_text.extend(core::iter::repeat_n(' ', base));
    root_text.push_str(&block.source);
    let root = SourceRoot::new(&root_text).expect("the root fits u32 offsets");
    let source = &root_text[base..];
    let frame = root
        .block(source, block.base)
        .expect("the block is the root's own slice");
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let lowered = lower_source_block_with_caps(&allocator, &tree, &errors, frame, caps);
    Ok(LoweredBlock {
        surface: Page {
            schema_version: S1_PAGE_SCHEMA,
            text: full_text(&SurfacePage::of(&tree)),
        },
        semantic: Page {
            schema_version: S2_PAGE_SCHEMA,
            text: full_text(&S2Folio::of(&lowered.root.ops)),
        },
        diagnostics: lowered.diagnostics.iter().map(Diagnostic::from).collect(),
    })
}
