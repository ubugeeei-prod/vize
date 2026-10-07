//! Keep native value provenance until authored shorthand edits are expanded.

use std::ops::Range;

use tower_lsp::lsp_types::{Location, WorkspaceEdit};
use vize_l0::{FxHashMap, String};

use super::super::CanonicalFailure;
use crate::ide::{IdeContext, corsa_support::CanonicalVirtualDocument};

pub(super) struct ShorthandRoles {
    properties: Vec<Location>,
    values: Vec<Location>,
    cache: FxHashMap<String, Option<Vec<Range<usize>>>>,
}

impl ShorthandRoles {
    pub(super) fn new(properties: Vec<Location>) -> Self {
        Self {
            properties,
            values: Vec::new(),
            cache: FxHashMap::default(),
        }
    }

    pub(super) fn capture(
        &mut self,
        ctx: &IdeContext<'_>,
        document: &CanonicalVirtualDocument,
        edit: Option<&WorkspaceEdit>,
    ) -> Result<(), CanonicalFailure> {
        if let Some(edit) = edit {
            self.values.extend(
                document
                    .selected_shorthand_value_arguments(
                        ctx,
                        edit,
                        &self.properties,
                        &mut self.cache,
                    )
                    .ok_or(CanonicalFailure::UnmappedResponse("shorthand value role"))?,
            );
        }
        Ok(())
    }

    pub(super) fn rewrite(
        &self,
        ctx: &IdeContext<'_>,
        document: &CanonicalVirtualDocument,
        edit: &mut WorkspaceEdit,
        new_name: &str,
    ) -> bool {
        if self.values.is_empty() {
            super::same_name_bindings::rewrite(ctx, document, edit, new_name, &self.properties)
        } else {
            super::same_name_bindings::rewrite_roles(
                ctx,
                document,
                edit,
                new_name,
                &self.properties,
                &self.values,
            )
        }
    }
}
