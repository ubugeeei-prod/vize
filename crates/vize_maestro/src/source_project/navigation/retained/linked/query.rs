//! Readonly request-time CST traversal: no source scanner or resident node index.
use super::super::super::{NavigationRefusal, SourceSnapshot, coordinates};
use tower_lsp::lsp_types::{LinkedEditingRanges, Position};
use vize_l0::Span;
use vize_l1::container::vue::NativeTemplateFrameNames;
use vize_l1::markup::{NativeElementClosingName, NativeTemplateComponent};

pub(super) struct TemplateNames<'o, 'a> {
    pub(super) snapshot: &'o SourceSnapshot,
    pub(super) selected: &'o NativeTemplateComponent<'a>,
    pub(super) frame: NativeTemplateFrameNames<'o, 'a>,
    pub(super) lines: &'o [usize],
    #[cfg(test)]
    pub(super) original: super::super::super::worker::linked::Inspection,
}
impl TemplateNames<'_, '_> {
    pub(super) fn ranges(
        &self,
        position: Position,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let offset = coordinates::offset(self.snapshot.source(), self.lines, position)?;
        let offset = u32::try_from(offset).map_err(|_| NavigationRefusal::Position)?;
        let block = self.selected.component().block();
        let opening = self.frame.opening();
        let closing = self.frame.closing();
        let frame_hit = caret(opening, offset) || caret(closing, offset);
        if (offset < block.start() || offset > block.end()) && !frame_hit {
            return Ok(None);
        }
        let mut result = if frame_hit {
            self.pair(opening, closing)?
        } else {
            None
        };
        // Iterative direct original-child traversal avoids unbounded recursion.
        // This temporary stack holds child iterators by depth, never node rows.
        let mut pending = vec![self.selected.children()];
        while let Some(children) = pending.last_mut() {
            if cancelled() {
                return Err(NavigationRefusal::Host(
                    super::super::super::SnapshotRefusal::Cancelled,
                ));
            }
            let Some(child) = children.next() else {
                pending.pop();
                continue;
            };
            let Some(element) = child.into_element() else {
                continue;
            };
            let names = element.names().map_err(NavigationRefusal::ElementNames)?;
            if let NativeElementClosingName::Present(close) = names.closing()
                && (caret(names.opening(), offset) || caret(close, offset))
            {
                if result.is_some() {
                    return Err(NavigationRefusal::Projection);
                }
                result = self.pair(names.opening(), close)?;
            }
            pending.push(element.children());
        }
        Ok(result)
    }

    fn pair(
        &self,
        opening: Span,
        closing: Span,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let source = self.snapshot.source();
        let open_text = source
            .get(opening.start as usize..opening.end as usize)
            .ok_or(NavigationRefusal::Projection)?;
        let close_text = source
            .get(closing.start as usize..closing.end as usize)
            .ok_or(NavigationRefusal::Projection)?;
        // Identical authored text is narrower than original ASCII case matching.
        // A case-mismatched frame still finishes the original CST traversal.
        if open_text != close_text {
            return Ok(None);
        }
        Ok(Some(LinkedEditingRanges {
            ranges: vec![
                coordinates::range(source, self.lines, opening)?,
                coordinates::range(source, self.lines, closing)?,
            ],
            word_pattern: None,
        }))
    }
}
// LSP linked editing includes the insertion caret just after a genuine name.
fn caret(span: Span, offset: u32) -> bool {
    span.start <= offset && offset <= span.end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentStore;
    use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
    use core::cell::Cell;
    use tower_lsp::lsp_types::Url;
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
    };
    use vize_l1::{
        SurfaceParseOptions,
        container::{Vue, vue::DescriptorOptions},
    };

    #[test]
    fn original_frame_hit_even_with_different_case_finishes_the_cancellable_cst_walk() {
        for source in [
            "<template><p></p></template>",
            "<template><p></p></TeMPLATE >",
        ] {
            let store = DocumentStore::new();
            let uri = Url::parse("file:///App.vue").unwrap();
            store.open(uri.clone(), source.into(), 1, "vue".into());
            let snapshot = SourceSnapshotCache::default()
                .capture(&store, &uri)
                .unwrap();
            let arena = Allocator::default();
            let descriptor = Vue.observe_descriptor(
                &arena,
                snapshot.source(),
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: SurfaceParseOptions::default(),
                },
            );
            let admitted = descriptor.admitted().unwrap();
            let frame = admitted.template().unwrap().frame_names().unwrap();
            let selected = NativeTemplateComponent::parse_in(&arena, admitted)
                .unwrap()
                .unwrap();
            let lines = vize_l0::line_index::LineBreaks::Lsp
                .line_starts(snapshot.source())
                .collect::<Vec<_>>();
            let query = TemplateNames {
                snapshot: &snapshot,
                selected: &selected,
                frame,
                lines: &lines,
                original: super::super::super::super::worker::linked::Inspection {
                    descriptor: core::ptr::from_ref(&descriptor) as usize,
                    selected: core::ptr::from_ref(&selected) as usize,
                    component: core::ptr::from_ref(selected.component()) as usize,
                    productions: 1,
                },
            };
            let visits = Cell::new(0);
            assert_eq!(
                query.ranges(Position::new(0, 1), || {
                    visits.set(visits.get() + 1);
                    true
                }),
                Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
            );
            assert_eq!(
                visits.get(),
                1,
                "neither a seeded pair nor case-mismatched null bypasses traversal"
            );
            assert!(query.frame.accepts(&admitted.template().unwrap()));
            assert_eq!(query.frame.container_index(), selected.template_index());
        }
    }

    #[test]
    fn original_frame_keeps_later_body_traversal_cancellation_before_pair_publication() {
        let source = "<template><p></p><div></div></template>";
        let store = DocumentStore::new();
        let uri = Url::parse("file:///App.vue").unwrap();
        store.open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&store, &uri)
            .unwrap();
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            snapshot.source(),
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let admitted = descriptor.admitted().unwrap();
        let frame = admitted.template().unwrap().frame_names().unwrap();
        let selected = NativeTemplateComponent::parse_in(&arena, admitted)
            .unwrap()
            .unwrap();
        let lines = vize_l0::line_index::LineBreaks::Lsp
            .line_starts(snapshot.source())
            .collect::<Vec<_>>();
        let query = TemplateNames {
            snapshot: &snapshot,
            selected: &selected,
            frame,
            lines: &lines,
            original: super::super::super::super::worker::linked::Inspection {
                descriptor: core::ptr::from_ref(&descriptor) as usize,
                selected: core::ptr::from_ref(&selected) as usize,
                component: core::ptr::from_ref(selected.component()) as usize,
                productions: 1,
            },
        };
        let visits = Cell::new(0);
        assert_eq!(
            query.ranges(Position::new(0, 1), || {
                visits.set(visits.get() + 1);
                visits.get() == 3
            }),
            Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled))
        );
        assert_eq!(
            visits.get(),
            3,
            "later original traversal remains authoritative after candidate creation"
        );
    }
}
