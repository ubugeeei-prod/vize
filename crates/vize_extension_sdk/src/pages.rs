//! Writers for the pages a host accepts.
//!
//! A host refuses a page that is not canonical (`print(parse(text)) ==
//! text`), so guests do not hand-format pages: [`s1::SurfacePage`] writes the
//! lossless S1 page (`s1-page@1`, offsets tiling the block) and
//! [`s2::SemanticPage`] writes the S2 disegno page (`s2-page@1`) for the op
//! subset a template dialect starts with — elements, attributes and text.

use alloc::string::String;

use crate::types::Page;
use crate::{S1_PAGE_SCHEMA, S2_PAGE_SCHEMA};

pub mod s1;
pub mod s2;

/// Wrap a written S1 page.
#[must_use]
pub fn s1_page(text: String) -> Page {
    Page {
        schema_version: S1_PAGE_SCHEMA,
        text,
    }
}

/// Wrap a written S2 page.
#[must_use]
pub fn s2_page(text: String) -> Page {
    Page {
        schema_version: S2_PAGE_SCHEMA,
        text,
    }
}
