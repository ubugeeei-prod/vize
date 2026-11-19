//! The HTML table insertion modes, entered only under
//! [`Profile::TABLE_CONTENT_MODEL`].
//!
//! Authored table parts close each other as the browser does: a row group
//! ends the previous one, a row the previous row, a cell the previous cell.
//! Transitions the browser cannot express with authored tags refuse: an
//! implied `tbody`, `tr` or `colgroup` start, fostered content and text, and
//! ignored tags or parse-error ends.

use super::{Builder, End, Mode, Refusal, Tag};
use crate::markup::Profile;

impl<P: Profile> Builder<'_, '_, P> {
    pub(super) fn table_start(&mut self, tag: Tag) -> Result<(), Refusal> {
        match (self.mode, tag.name) {
            (Mode::InTable, "caption") => self.enter(tag, &["table"], Mode::InCaption),
            (Mode::InTable, "colgroup") => self.enter(tag, &["table"], Mode::InColumnGroup),
            (Mode::InTable, "tbody" | "thead" | "tfoot") => {
                self.enter(tag, &["table"], Mode::InTableBody)
            }
            (Mode::InTable, "col" | "tr" | "td" | "th") => Err(Refusal::ImpliedStart(tag.span)),
            (Mode::InTable | Mode::InTableBody | Mode::InRow, "table") => {
                Err(Refusal::ImpliedEnd(tag.span))
            }
            (Mode::InTableBody, "tr") => self.enter(tag, &ROW_GROUPS, Mode::InRow),
            (Mode::InTableBody, "td" | "th") => Err(Refusal::ImpliedStart(tag.span)),
            (Mode::InTableBody, "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead") => {
                self.leave(&ROW_GROUPS, tag.span, tag.event, Mode::InTable)?;
                self.table_start(tag)
            }
            (Mode::InRow, "td" | "th") => self.enter(tag, &["tr"], Mode::InCell),
            (Mode::InRow, "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead" | "tr") => {
                self.leave(&["tr"], tag.span, tag.event, Mode::InTableBody)?;
                self.table_start(tag)
            }
            (
                Mode::InCell,
                "caption" | "col" | "colgroup" | "tbody" | "td" | "tfoot" | "th" | "thead" | "tr",
            ) => {
                if !self.in_table_scope(&["td", "th"]) {
                    return Err(Refusal::IgnoredTag(tag.span));
                }
                self.close_cell(tag.span, tag.event)?;
                self.table_start(tag)
            }
            (
                Mode::InCaption,
                "caption" | "col" | "colgroup" | "tbody" | "td" | "tfoot" | "th" | "thead" | "tr",
            ) => {
                self.close_caption(tag.span, None, tag.event)?;
                self.table_start(tag)
            }
            (Mode::InCell | Mode::InCaption, _) => self.body_start(tag),
            (Mode::InColumnGroup, "col") => self.insert(tag),
            (Mode::InColumnGroup, _) => {
                self.leave_column_group(tag.span, tag.event)?;
                self.table_start(tag)
            }
            _ => Err(Refusal::FosterParenting(tag.span)),
        }
    }

    pub(super) fn table_end(&mut self, end: &End) -> Result<(), Refusal> {
        match (self.mode, end.name) {
            (Mode::InTable, "table") => {
                if !self.in_table_scope(&["table"]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.pop_matching(end)?;
                self.reset_mode();
                Ok(())
            }
            (Mode::InTableBody, "tbody" | "tfoot" | "thead") => {
                if !self.in_table_scope(&[end.name]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.clear_to(&ROW_GROUPS, end.event)?;
                self.pop_matching(end)?;
                self.mode = Mode::InTable;
                Ok(())
            }
            (Mode::InTableBody, "table") => {
                self.leave(&ROW_GROUPS, end.span, end.event, Mode::InTable)?;
                self.table_end(end)
            }
            (Mode::InRow, "tr") => {
                if !self.in_table_scope(&["tr"]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.clear_to(&["tr"], end.event)?;
                self.pop_matching(end)?;
                self.mode = Mode::InTableBody;
                Ok(())
            }
            (Mode::InRow, "table" | "tbody" | "tfoot" | "thead") => {
                if end.name != "table" && !self.in_table_scope(&[end.name]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.leave(&["tr"], end.span, end.event, Mode::InTableBody)?;
                self.table_end(end)
            }
            (Mode::InCell, "td" | "th") => {
                if !self.in_table_scope(&[end.name]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.implied_ends(None, end.event)?;
                self.pop_matching(end)?;
                self.mode = Mode::InRow;
                Ok(())
            }
            (Mode::InCell, "table" | "tbody" | "tfoot" | "thead" | "tr") => {
                if !self.in_table_scope(&[end.name]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.close_cell(end.span, end.event)?;
                self.table_end(end)
            }
            (Mode::InCaption, "caption") => self.close_caption(end.span, Some(end), end.event),
            (Mode::InCaption, "table") => {
                self.close_caption(end.span, None, end.event)?;
                self.table_end(end)
            }
            (
                Mode::InCell | Mode::InCaption,
                "body" | "caption" | "col" | "colgroup" | "html" | "tbody" | "td" | "tfoot" | "th"
                | "thead" | "tr",
            ) => Err(Refusal::ImpliedEnd(end.span)),
            (Mode::InCell | Mode::InCaption, _) => self.body_end(end),
            (Mode::InColumnGroup, "colgroup") => {
                self.pop_matching(end)?;
                self.mode = Mode::InTable;
                Ok(())
            }
            (Mode::InColumnGroup, "col") => Err(Refusal::ImpliedEnd(end.span)),
            (Mode::InColumnGroup, _) => {
                self.leave_column_group(end.span, end.event)?;
                self.table_end(end)
            }
            // Misplaced table parts are ignored; other ends would close
            // nothing inside the table or act on fostered content.
            _ => Err(Refusal::ImpliedEnd(end.span)),
        }
    }

    /// Clear back to one of `contexts`, then insert a table part.
    fn enter(&mut self, tag: Tag, contexts: &[&str], mode: Mode) -> Result<(), Refusal> {
        self.clear_to(contexts, tag.event)?;
        self.insert(tag)?;
        self.mode = mode;
        Ok(())
    }

    /// Imply the end of the open row group or row in table scope.
    fn leave(
        &mut self,
        parts: &[&str],
        span: vize_l0::Span,
        event: usize,
        mode: Mode,
    ) -> Result<(), Refusal> {
        if !self.in_table_scope(parts) {
            return Err(Refusal::IgnoredTag(span));
        }
        self.clear_to(parts, event)?;
        self.pop(None, event)?;
        self.mode = mode;
        Ok(())
    }

    fn leave_column_group(&mut self, span: vize_l0::Span, event: usize) -> Result<(), Refusal> {
        if self.current() != Some("colgroup") {
            return Err(Refusal::IgnoredTag(span));
        }
        self.pop(None, event)?;
        self.mode = Mode::InTable;
        Ok(())
    }

    fn close_cell(&mut self, span: vize_l0::Span, event: usize) -> Result<(), Refusal> {
        self.implied_ends(None, event)?;
        if !matches!(self.current(), Some("td" | "th")) {
            return Err(Refusal::ImpliedEnd(span));
        }
        self.pop(None, event)?;
        self.mode = Mode::InRow;
        Ok(())
    }

    fn close_caption(
        &mut self,
        span: vize_l0::Span,
        end: Option<&End>,
        event: usize,
    ) -> Result<(), Refusal> {
        if !self.in_table_scope(&["caption"]) {
            return Err(Refusal::ImpliedEnd(span));
        }
        self.implied_ends(None, event)?;
        match end {
            Some(end) => self.pop_matching(end)?,
            None if self.current() == Some("caption") => self.pop(None, event)?,
            None => return Err(Refusal::ImpliedEnd(span)),
        }
        self.mode = Mode::InTable;
        Ok(())
    }

    /// Pop until the current node is one of `contexts` (or `html`).
    fn clear_to(&mut self, contexts: &[&str], event: usize) -> Result<(), Refusal> {
        while let Some(name) = self.current()
            && name != "html"
            && !contexts.contains(&name)
        {
            self.pop(None, event)?;
        }
        Ok(())
    }

    fn in_table_scope(&self, targets: &[&str]) -> bool {
        for depth in (0..self.stack.len()).rev() {
            let Some(node) = self.name_at(depth) else {
                return false;
            };
            if targets.contains(&node) {
                return true;
            }
            if matches!(node, "html" | "table") {
                return false;
            }
        }
        false
    }

    /// Reset the insertion mode from the open elements after a table closes.
    fn reset_mode(&mut self) {
        for depth in (0..self.stack.len()).rev() {
            let mode = match self.name_at(depth) {
                Some("td" | "th") => Mode::InCell,
                Some("tr") => Mode::InRow,
                Some("tbody" | "thead" | "tfoot") => Mode::InTableBody,
                Some("caption") => Mode::InCaption,
                Some("colgroup") => Mode::InColumnGroup,
                Some("table") => Mode::InTable,
                Some("body") | None => Mode::InBody,
                Some(_) => continue,
            };
            self.mode = mode;
            return;
        }
        self.mode = Mode::InBody;
    }
}

const ROW_GROUPS: [&str; 3] = ["tbody", "thead", "tfoot"];
