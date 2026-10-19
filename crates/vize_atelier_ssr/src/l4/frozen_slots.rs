//! Private custody for the original admitted empty literal v-pre slot.
//!
//! This trusts the compiler's original parser bundle and its freshly lowered
//! template artifact. It is not an authentication scheme for public mutable
//! ASTs. Direct caller-built L2 receives no fact, even for identical offsets.

use vize_armature::parser::FrozenElements;
use vize_l0::Span;
use vize_l2::op::{ElementOp, Region};
use vize_relief::RootNode;

const FOREIGN_SOURCE: &str =
    "Frozen literal slot provenance belongs to a different template source.";
const FOREIGN_ARTIFACT: &str = "Frozen literal slot facts belong to a different L2 artifact.";

/// A validated borrowed view of the existing parser sidecar; no allocation.
#[derive(Clone, Copy)]
pub(super) struct FrozenTemplate<'a> {
    source: &'a str,
    spans: &'a [Span],
}

impl<'a> FrozenTemplate<'a> {
    pub(super) fn new(
        root: &RootNode<'a>,
        source: &'a str,
        frozen: &FrozenElements<'a>,
    ) -> Result<Self, &'static str> {
        let spans = frozen
            .spans_for(root.source, source)
            .ok_or(FOREIGN_SOURCE)?;
        Ok(Self { source, spans })
    }

    pub(super) fn lower(
        self,
        allocator: &'a vize_l0::Allocator,
        tree: &vize_l1::SurfaceTree<'a>,
        errors: &[vize_l1::SurfaceError],
    ) -> Result<vize_l1_to_l2::Lowered<'a>, &'static str> {
        let source = vize_l0::SourceRoot::new(self.source).map_err(|_| FOREIGN_SOURCE)?;
        vize_l1_to_l2::lower_with_frozen_element_spans(
            allocator,
            tree,
            errors,
            source.whole_block(),
            self.spans,
        )
    }
}

/// Only the source bridge mints this alongside its actual lowered Region.
/// The selector validates that same artifact before any lowering or context.
pub(super) struct FrozenSlotFacts<'s, 'a> {
    source: &'a str,
    root: &'s Region<'a>,
    spans: &'s [Span],
}

impl<'s, 'a> FrozenSlotFacts<'s, 'a> {
    pub(super) fn new(
        template: FrozenTemplate<'a>,
        source: &'a str,
        root: &'s Region<'a>,
    ) -> Result<Self, &'static str> {
        if !core::ptr::eq(template.source, source) {
            return Err(FOREIGN_SOURCE);
        }
        Ok(Self {
            source,
            root,
            spans: template.spans,
        })
    }

    pub(super) fn validate(&self, source: &str, root: &Region<'_>) -> Result<(), &'static str> {
        if !core::ptr::eq(self.source, source) || !core::ptr::eq(self.root, root) {
            return Err(FOREIGN_ARTIFACT);
        }
        Ok(())
    }

    /// Called for the actual ElementOp borrowed by the validated string plan.
    /// Opening custody comes from Parser, while ElementOp.span includes its
    /// closing tag. Compare the shared start and bounds without scanning again.
    pub(super) fn permits(&self, element: &ElementOp<'_>) -> bool {
        if element.tag != "slot"
            || !element.bindings.is_empty()
            || !element.children.ops.is_empty()
            || element.span.start >= element.span.end
            || element.span.end as usize > self.source.len()
        {
            return false;
        }
        let Ok(index) = self
            .spans
            .binary_search_by_key(&element.span.start, |span| span.start)
        else {
            return false;
        };
        let Some(opening) = self.spans.get(index) else {
            return false;
        };
        let Some(start) = (opening.start as usize).checked_add(1) else {
            return false;
        };
        let Some(end) = start.checked_add(element.tag.len()) else {
            return false;
        };
        opening.end <= element.span.end
            && end <= opening.end as usize
            && self
                .source
                .get(start..end)
                .is_some_and(|tag| core::ptr::eq(tag, element.tag))
    }
}

#[cfg(test)]
mod tests {
    //! Authored before the source-bound empty literal-slot exception.

    use super::{FrozenSlotFacts, FrozenTemplate};
    use crate::l4::{LegacyReason, SsrL4Selection, differential_tests, l2_input};
    use crate::options::SsrCompilerOptions;
    use vize_atelier_core::parser::Parser;
    use vize_l0::{Allocator, Box, Span, Vec};
    use vize_l2::op::{ElementOp, Namespace, Op, Region};

    fn lower<'a>(arena: &'a Allocator, source: &'a str) -> vize_l1_to_l2::lower::Lowered<'a> {
        let (tree, errors) = vize_l1::parse(arena, source);
        assert!(errors.is_empty(), "{errors:?}");
        let (root, parser_errors, carrier) =
            Parser::new(arena, source).parse_with_frozen_elements();
        assert!(parser_errors.is_empty(), "{parser_errors:?}");
        FrozenTemplate::new(&root, source, &carrier)
            .expect("original parser bundle")
            .lower(arena, &tree, &errors)
            .expect("complete opening custody")
    }

    #[test]
    fn frozen_template_rejects_foreign_actual_root_and_caller_before_mutation() {
        for case in 0..3 {
            let arena = Allocator::new();
            let source_a = "<slot v-pre></slot>".to_owned();
            let source_b = if case == 2 {
                source_a.clone()
            } else {
                "<slot></slot>".to_owned()
            };
            let (root_a, errors, frozen) =
                Parser::new(&arena, &source_a).parse_with_frozen_elements();
            assert!(errors.is_empty(), "{errors:?}");
            let (root_b, errors) = Parser::new(&arena, &source_b).parse();
            assert!(errors.is_empty(), "{errors:?}");
            let (root, caller) = if case == 1 {
                (&root_a, source_b.as_str())
            } else {
                (&root_b, source_a.as_str())
            };
            let before = vize_l0::cstr!("{root:?}");
            assert_eq!(
                FrozenTemplate::new(root, caller, &frozen).err(),
                Some("Frozen literal slot provenance belongs to a different template source.")
            );
            assert_eq!(vize_l0::cstr!("{root:?}"), before, "case {case}");
        }
    }

    #[test]
    fn capability_is_bound_to_the_actual_lowered_region_and_source() {
        let arena = Allocator::new();
        let source = "<slot v-pre></slot>".to_owned();
        let copy = source.clone();
        let (root, errors, frozen) = Parser::new(&arena, &source).parse_with_frozen_elements();
        assert!(errors.is_empty(), "{errors:?}");
        let template =
            FrozenTemplate::new(&root, &source, &frozen).expect("original parser bundle");
        let owned = lower(&arena, &source);
        let equivalent = lower(&arena, &source);
        let facts = FrozenSlotFacts::new(template, &source, &owned.root).expect("owned lowering");
        assert!(facts.validate(&source, &owned.root).is_ok());
        for (caller, region) in [
            (source.as_str(), &equivalent.root),
            (copy.as_str(), &owned.root),
        ] {
            let before = vize_l0::cstr!("{region:?}");
            assert_eq!(
                facts.validate(caller, region),
                Err("Frozen literal slot facts belong to a different L2 artifact.")
            );
            assert_eq!(vize_l0::cstr!("{region:?}"), before);
        }
        assert_eq!(
            FrozenSlotFacts::new(template, &copy, &owned.root).err(),
            Some("Frozen literal slot provenance belongs to a different template source.")
        );
        let Op::Element(slot) = &owned.root.ops[0] else {
            panic!("frozen literal is ElementOp")
        };
        assert!(facts.permits(slot));
    }

    #[test]
    fn direct_l2_cannot_forge_literal_slot_custody_from_offsets_or_tag_text() {
        let arena = &Allocator::new();
        let source = "<slot v-pre></slot>";
        let detached = "slot".to_owned();
        for tag in [&source[1..5], detached.as_str()] {
            let element = ElementOp {
                tag,
                namespace: Namespace::Html,
                attributes: Vec::new_in(&arena),
                bindings: Vec::new_in(&arena),
                children: Region {
                    ops: Vec::new_in(&arena),
                },
                span: Span::new(0, source.len() as u32),
            };
            let mut root = Region {
                ops: Vec::new_in(&arena),
            };
            root.ops.push(Op::Element(Box::new_in(element, &arena)));
            let options = SsrCompilerOptions::default();
            assert!(matches!(
                l2_input::select_l2_lane(&arena, source, &root, &options),
                SsrL4Selection::Legacy(LegacyReason::Element)
            ));
            assert!(crate::compile_l2_to_ssr(&arena, source, &root, &options).is_none());
        }
        let normal = lower(&arena, "<slot></slot>");
        assert!(matches!(&normal.root.ops[0], Op::Slot(_)));
        assert!(matches!(
            l2_input::select_l2_lane(
                &arena,
                "<slot></slot>",
                &normal.root,
                &SsrCompilerOptions::default()
            ),
            SsrL4Selection::Emitted(_)
        ));
    }

    #[test]
    fn original_empty_literal_and_normal_outlet_admit_while_nonempty_literal_refuses() {
        for (name, source, admitted) in [
            ("own-empty", "<slot v-pre></slot>", true),
            ("inherited-empty", "<div v-pre><slot></slot></div>", true),
            ("normal-outlet", "<slot></slot>", true),
            (
                "original-refused",
                "<slot v-pre>{{ not }} an interpolation</slot>",
                false,
            ),
        ] {
            for (set, options, experimental) in differential_tests::option_sets() {
                let selection = differential_tests::selection(source, &options, &experimental);
                if admitted {
                    assert!(
                        matches!(selection, SsrL4Selection::Emitted(_)),
                        "{name}/{set}: {selection:?}"
                    );
                } else {
                    assert!(
                        matches!(selection, SsrL4Selection::Legacy(LegacyReason::Element)),
                        "{name}/{set}: {selection:?}"
                    );
                }
                differential_tests::assert_parity(source, &options, &experimental, name);
            }
        }
    }

    #[test]
    fn recovered_provider_mismatches_refuse_complete_custody_without_root_mutation() {
        for (source, message) in [
            (
                "<p v-pre><div><Child></Child></div><p></p>",
                "Frozen lowering custody is missing an authored frozen opening.",
            ),
            (
                "<button v-pre><button></button></button>",
                "Frozen lowering custody contains an unmatched or non-frozen opening.",
            ),
            (
                "<form v-pre><form><Child></Child></form></form>",
                "Frozen lowering custody is missing an authored frozen opening.",
            ),
        ] {
            let arena = Allocator::new();
            let (root, _parser_errors, carrier) =
                Parser::new(&arena, source).parse_with_frozen_elements();
            let before = vize_l0::cstr!("{root:#?}");
            let (tree, errors) = vize_l1::parse(&arena, source);
            let tree_before = vize_l0::cstr!("{tree:#?}");
            let template =
                FrozenTemplate::new(&root, source, &carrier).expect("actual original parser join");
            assert_eq!(
                template.lower(&arena, &tree, &errors).err(),
                Some(message),
                "{source}"
            );
            assert_eq!(vize_l0::cstr!("{root:#?}"), before);
            assert_eq!(vize_l0::cstr!("{tree:#?}"), tree_before);
        }
    }
}
