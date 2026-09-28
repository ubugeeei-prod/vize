//! Writers for the pages a host accepts.
//!
//! A host refuses a page that is not canonical (`print(parse(text)) ==
//! text`), so guests do not hand-format pages: [`l1::SurfacePage`] writes the
//! lossless L1 page (`s1-page@1`, offsets tiling the block) and
//! [`l2::SemanticPage`] writes the L2 disegno page (`s2-page@1`) for the op
//! subset a template dialect starts with — elements, attributes and text.

use alloc::string::String;

use crate::types::Page;
use crate::{L1_PAGE_SCHEMA, L2_PAGE_SCHEMA};

pub mod l1;
pub mod l2;

/// Compatibility names for the original SDK modules and page wrappers.
pub use self::{l1 as s1, l1_page as s1_page, l2 as s2, l2_page as s2_page};

/// Wrap a written L1 page.
#[must_use]
pub fn l1_page(text: String) -> Page {
    Page {
        schema_version: L1_PAGE_SCHEMA,
        text,
    }
}

/// Wrap a written L2 page.
#[must_use]
pub fn l2_page(text: String) -> Page {
    Page {
        schema_version: L2_PAGE_SCHEMA,
        text,
    }
}
