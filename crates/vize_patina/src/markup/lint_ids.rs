//! Static ID facts with the established document-rule branch contract.
//!
//! Template facts borrow the normal L1 tree: ignored starts are flattened,
//! frozen directives stay inert and every entity scalar is decoded. Embedded
//! markup keeps its lowered scopes. Duplicate detection lives in the rule.

use super::{MarkupDocument, MarkupDocumentInner};
use crate::ir::ByteRange;
use vize_l0::{String, ToCompactString};

mod relief;
mod surface;

pub(crate) struct LintStaticId {
    pub(crate) value: String,
    pub(crate) range: ByteRange,
    branches: Vec<(u32, usize)>,
}

impl LintStaticId {
    pub(crate) fn can_coexist(&self, other: &Self) -> bool {
        self.branches.iter().all(|(start, index)| {
            other
                .branches
                .iter()
                .find(|(other_start, _)| other_start == start)
                .is_none_or(|(_, other_index)| other_index == index)
        })
    }
}

impl MarkupDocument<'_> {
    pub(crate) fn lint_static_ids(&self) -> Vec<LintStaticId> {
        let mut ids = Vec::new();
        match self.inner {
            MarkupDocumentInner::L2(doc) if doc.surface.is_some() => {
                if let Some(tree) = doc.surface {
                    surface::collect(tree, &mut ids);
                }
            }
            MarkupDocumentInner::Relief(root) => {
                relief::collect(&root.children, &mut Vec::new(), &mut ids);
            }
            _ => self.walk_branched_elements(&mut |element, branches| {
                element.walk_attributes(&mut |attribute| {
                    if attribute.is_plain_static()
                        && attribute.name() == "id"
                        && let Some(value) = attribute.value()
                    {
                        ids.push(LintStaticId {
                            value: value.to_compact_string(),
                            range: attribute.range(),
                            branches: branches.iter().map(|branch| branch.lint_choice()).collect(),
                        });
                    }
                });
            }),
        }
        ids
    }
}
