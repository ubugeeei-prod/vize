//! Table foster parenting and implicit table section/row insertion.

use vize_carton::Vec;
use vize_relief::*;

use super::super::{Parser, ParserStackEntry, StackInsertion};

impl<'a> Parser<'a> {
    pub(in crate::parser) fn nearest_table_index(&self) -> Option<usize> {
        if self.open_table_count == 0 {
            return None;
        }

        (0..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].element.tag.eq_ignore_ascii_case("table"))
    }

    pub(super) fn should_foster_text(&self, content: &str) -> bool {
        self.open_table_count > 0
            && content
                .chars()
                .any(|c| !matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000C}'))
            && self.is_in_table_insertion_context()
    }

    pub(super) fn should_foster_start_tag(&self, tag: &str, is_html_element: bool) -> bool {
        is_html_element
            && self.open_table_count > 0
            && self.is_in_table_insertion_context()
            && !self.is_allowed_in_table_insertion_context(tag)
    }

    fn is_in_table_insertion_context(&self) -> bool {
        let Some(table_index) = self.nearest_table_index() else {
            return false;
        };

        for entry in self.stack.iter().skip(table_index + 1) {
            if entry.insertion == StackInsertion::Fostered
                || Self::tag_in(entry.element.tag.as_str(), &["caption", "td", "th"])
            {
                return false;
            }
        }

        true
    }

    fn is_allowed_in_table_insertion_context(&self, tag: &str) -> bool {
        let current = self.stack.last().map(|entry| entry.element.tag.as_str());
        match current {
            Some(current) if Self::tag_in(current, &["tbody", "thead", "tfoot"]) => {
                Self::tag_in(tag, &["tr", "script", "style", "template"])
            }
            Some(current) if current.eq_ignore_ascii_case("tr") => {
                Self::tag_in(tag, &["td", "th", "script", "style", "template"])
            }
            _ => Self::tag_in(
                tag,
                &[
                    "caption", "colgroup", "col", "tbody", "thead", "tfoot", "tr", "td", "th",
                    "script", "style", "template",
                ],
            ),
        }
    }

    pub(super) fn handle_in_table_start_tag(&mut self, tag: &str, offset: usize) {
        if self.open_table_count == 0 {
            return;
        }

        if !self.is_in_table_insertion_context() {
            return;
        }

        if Self::tag_in(tag, &["tbody", "thead", "tfoot"])
            && self.stack.last().is_some_and(|entry| {
                Self::tag_in(entry.element.tag.as_str(), &["tbody", "thead", "tfoot"])
            })
        {
            let last = self.stack.len() - 1;
            self.close_stack_element_at(last, false);
        }

        if tag.eq_ignore_ascii_case("tr") {
            if self
                .stack
                .last()
                .is_some_and(|entry| entry.element.tag.eq_ignore_ascii_case("tr"))
            {
                let last = self.stack.len() - 1;
                self.close_stack_element_at(last, false);
            }
            self.ensure_table_section(offset);
        } else if Self::tag_in(tag, &["td", "th"]) {
            if self
                .stack
                .last()
                .is_some_and(|entry| Self::tag_in(entry.element.tag.as_str(), &["td", "th"]))
            {
                let last = self.stack.len() - 1;
                self.close_stack_element_at(last, false);
            }
            self.ensure_table_section(offset);
            self.ensure_table_row(offset);
        }
    }

    fn ensure_table_section(&mut self, offset: usize) {
        let Some(table_index) = self.nearest_table_index() else {
            return;
        };
        if self
            .stack
            .iter()
            .skip(table_index + 1)
            .any(|entry| Self::tag_in(entry.element.tag.as_str(), &["tbody", "thead", "tfoot"]))
        {
            return;
        }
        self.push_implicit_element("tbody", offset);
    }

    fn ensure_table_row(&mut self, offset: usize) {
        if self
            .stack
            .last()
            .is_some_and(|entry| entry.element.tag.eq_ignore_ascii_case("tr"))
        {
            return;
        }
        self.push_implicit_element("tr", offset);
    }

    fn push_implicit_element(&mut self, tag: &str, offset: usize) {
        let loc = self.create_loc(offset, offset);
        let mut element = ElementNode::new(self.allocator, tag, loc);
        element.tag_type = self.determine_element_type(&element);
        self.push_stack_entry(ParserStackEntry {
            element,
            in_pre: self.in_pre,
            in_v_pre: self.in_v_pre,
            insertion: StackInsertion::Normal,
            implicit: true,
            fostered_before: Vec::new_in(self.allocator),
        });
    }
}
