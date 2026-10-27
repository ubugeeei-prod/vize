//! The "in body" insertion mode for the admitted element set.
//!
//! Under [`Profile::IMPLIED_END_TAGS`] start and end tags imply the HTML end
//! tags of `p`, `li`, `dt`, `dd`, ruby text and `option` parents. Only
//! error-free implied ends are admitted: any parse error these rules would
//! report refuses, so every closed element keeps its authored parent. Without
//! the policy an end tag must name the current node, as in a Vue template.

use super::{Builder, End, Mode, Refusal, Tag};
use crate::markup::Profile;

impl<P: Profile> Builder<'_, '_, P> {
    pub(super) fn body_start(&mut self, tag: Tag) -> Result<(), Refusal> {
        if P::TABLE_CONTENT_MODEL && TABLE_PARTS.contains(&tag.name) {
            return Err(Refusal::IgnoredTag(tag.span));
        }
        if P::IMPLIED_END_TAGS {
            match tag.name {
                "li" => self.close_list_item(&tag, &["li"])?,
                "dd" | "dt" => self.close_list_item(&tag, &["dd", "dt"])?,
                "rb" | "rtc" => self.close_ruby_text(&tag, None, &["ruby"])?,
                "rp" | "rt" => self.close_ruby_text(&tag, Some("rtc"), &["ruby", "rtc"])?,
                "option" | "optgroup" if self.current() == Some("option") => {
                    self.pop(None, tag.event)?;
                }
                _ => {}
            }
            if CLOSES_P.contains(&tag.name) && self.in_scope(&["p"], &["button"]) {
                self.close_p(&tag)?;
            }
        }
        let table = P::TABLE_CONTENT_MODEL && tag.name == "table";
        self.insert(tag)?;
        if table {
            self.mode = Mode::InTable;
        }
        Ok(())
    }

    pub(super) fn body_end(&mut self, end: &End) -> Result<(), Refusal> {
        if matches!(end.name, "html" | "head") {
            return Err(Refusal::ExplicitEnvelope);
        }
        if end.name == "body" {
            // The body stays open in the browser, and so do implied-end
            // descendants: later whitespace joins the current node.
            self.tail_owner = self.stack.last().copied();
            while P::IMPLIED_END_TAGS && self.current().is_some_and(|name| IMPLIED.contains(&name))
            {
                self.pop(None, end.event)?;
            }
            self.pop_matching(end)?;
            self.mode = Mode::HtmlEnd;
            return Ok(());
        }
        if !P::IMPLIED_END_TAGS || super::void(end.name) {
            return self.pop_matching(end);
        }
        match end.name {
            "p" => {
                if !self.in_scope(&["p"], &["button"]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.close_p_end(end)
            }
            "li" => self.close_scoped(end, &["ol", "ul"]),
            "dd" | "dt" => self.close_scoped(end, &[]),
            name if CLOSES_P.contains(&name) => {
                if !self.in_scope(&[name], &[]) {
                    return Err(Refusal::ImpliedEnd(end.span));
                }
                self.implied_ends(None, end.event)?;
                self.pop_matching(end)
            }
            _ => self.close_other(end),
        }
    }

    /// `li`, `dt` and `dd` close the nearest such item unless a special
    /// element other than `address`, `div` or `p` intervenes.
    fn close_list_item(&mut self, tag: &Tag, items: &[&str]) -> Result<(), Refusal> {
        for depth in (0..self.stack.len()).rev() {
            let Some(node) = self.name_at(depth) else {
                break;
            };
            if items.contains(&node) {
                self.implied_ends(Some(node), tag.event)?;
                if self.current() != Some(node) {
                    return Err(Refusal::ImpliedEnd(tag.span));
                }
                return self.pop(None, tag.event);
            }
            if special(node) && !matches!(node, "address" | "div" | "p") {
                break;
            }
        }
        Ok(())
    }

    fn close_ruby_text(
        &mut self,
        tag: &Tag,
        except: Option<&'static str>,
        parents: &[&str],
    ) -> Result<(), Refusal> {
        if self.in_scope(&["ruby"], &[]) {
            self.implied_ends(except, tag.event)?;
            if !self.current().is_some_and(|name| parents.contains(&name)) {
                return Err(Refusal::ImpliedEnd(tag.span));
            }
        }
        Ok(())
    }

    /// Close the open `p` in button scope for a start tag.
    fn close_p(&mut self, tag: &Tag) -> Result<(), Refusal> {
        self.implied_ends(Some("p"), tag.event)?;
        if self.current() != Some("p") {
            return Err(Refusal::ImpliedEnd(tag.span));
        }
        self.pop(None, tag.event)
    }

    fn close_p_end(&mut self, end: &End) -> Result<(), Refusal> {
        self.implied_ends(Some("p"), end.event)?;
        self.pop_matching(end)
    }

    /// `</li>`, `</dt>` and `</dd>`: the item must be in scope and current
    /// once its implied-end descendants close.
    fn close_scoped(&mut self, end: &End, extra: &[&str]) -> Result<(), Refusal> {
        if !self.in_scope(&[end.name], extra) {
            return Err(Refusal::ImpliedEnd(end.span));
        }
        self.implied_ends(Some(end.name), end.event)?;
        self.pop_matching(end)
    }

    /// "Any other end tag": the nearest same-named element closes unless a
    /// special element intervenes.
    fn close_other(&mut self, end: &End) -> Result<(), Refusal> {
        for depth in (0..self.stack.len()).rev() {
            let Some(node) = self.name_at(depth) else {
                break;
            };
            if node == end.name {
                self.implied_ends(Some(node), end.event)?;
                return self.pop_matching(end);
            }
            if special(node) {
                break;
            }
        }
        Err(Refusal::ImpliedEnd(end.span))
    }

    /// Generate implied end tags, except for `except`.
    pub(super) fn implied_ends(
        &mut self,
        except: Option<&str>,
        event: usize,
    ) -> Result<(), Refusal> {
        while let Some(name) = self.current()
            && IMPLIED.contains(&name)
            && Some(name) != except
        {
            self.pop(None, event)?;
        }
        Ok(())
    }

    /// An element in default scope, widened by `extra` boundaries.
    pub(super) fn in_scope(&self, targets: &[&str], extra: &[&str]) -> bool {
        for depth in (0..self.stack.len()).rev() {
            let Some(node) = self.name_at(depth) else {
                return false;
            };
            if targets.contains(&node) {
                return true;
            }
            if matches!(node, "html" | "table" | "caption" | "td" | "th") || extra.contains(&node) {
                return false;
            }
        }
        false
    }
}

/// Elements whose end tag the parser may imply.
pub(super) const IMPLIED: [&str; 10] = [
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
];

/// Table parts the "in body" mode ignores.
pub(super) const TABLE_PARTS: [&str; 9] = [
    "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr",
];

/// Admitted start tags that close an open `p` in button scope.
const CLOSES_P: [&str; 24] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "footer",
    "header",
    "hgroup",
    "hr",
    "li",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "search",
    "section",
    "table",
    "ul",
];

/// The HTML special category, restricted to the admitted names.
fn special(name: &str) -> bool {
    !matches!(
        name,
        "span" | "ruby" | "rb" | "rt" | "rtc" | "rp" | "option" | "optgroup"
    )
}
