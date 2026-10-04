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
mod tests;
