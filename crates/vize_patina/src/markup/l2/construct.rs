//! Constructors of [`L2Template`]: the template parse and lowering that
//! match how the template reaches Vue.

use super::L2Template;
use vize_l0::Allocator;
use vize_l1::markup::{SurfaceError, SurfaceTree};
use vize_l1_to_l2::lower::{LegacyCaps, LowerPolicy, lower_with_caps_and_comment_policy};

impl<'a> L2Template<'a> {
    /// Parse `source` into L1 and lower it into L2, keeping authored comments
    /// as `ui.comment` ops (the child list the lint parser sees).
    pub fn lower(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::from_parse(
            allocator,
            vize_l1::markup::parse_with_authored_diagnostics(allocator, source),
            LowerPolicy::lint(),
        )
    }

    /// [`Self::lower`] for a template the browser parses as HTML first
    /// (standalone HTML): its diagnostics also report foster parenting, and
    /// rows and cells written directly in a `<table>` gain the `tbody`/`tr`
    /// owners that parser inserts.
    pub fn lower_in_dom(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::from_parse(
            allocator,
            vize_l1::markup::parse_in_dom_with_authored_diagnostics(allocator, source),
            LowerPolicy::lint().with_implied_table_sections(),
        )
    }

    /// A standalone HTML document is parsed by the browser before Vue reads it,
    /// so its tables foster misplaced content and imply owners; a Vue template
    /// keeps its authored structure.
    pub(crate) fn lower_for(
        allocator: &'a Allocator,
        source: &'a str,
        kind: crate::context::TemplateDocumentKind,
    ) -> Self {
        match kind {
            crate::context::TemplateDocumentKind::StandaloneHtml => {
                Self::lower_in_dom(allocator, source)
            }
            crate::context::TemplateDocumentKind::VueTemplate => Self::lower(allocator, source),
        }
    }

    fn from_parse(
        allocator: &'a Allocator,
        (tree, authored, errors): (
            SurfaceTree<'a>,
            Option<SurfaceTree<'a>>,
            vize_l0::Vec<'a, SurfaceError>,
        ),
        policy: LowerPolicy,
    ) -> Self {
        let lowered = lower_with_caps_and_comment_policy(
            allocator,
            &tree,
            &errors,
            LegacyCaps::VUE3,
            policy,
            &[],
            None,
        );
        Self {
            tree,
            authored,
            errors,
            lowered,
        }
    }
}
