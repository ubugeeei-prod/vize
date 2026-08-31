//! Element processing methods for the parser.
//!
//! Handles text, interpolation, open/close tags, element type determination,
//! comments, and error reporting.

mod classify;
mod comment;
mod errors;
mod formatting;
mod nesting;
mod scope;
mod table;
mod tags;
mod text;

use vize_relief::{
    ElementNode, ElementType, Namespace, SourceLocation,
    errors::{CompilerError, ErrorCode},
};

use super::Parser;

impl<'a> Parser<'a> {
    fn report_tree_construction_recovery(&mut self, loc: &SourceLocation, message: &str) {
        self.errors.push(CompilerError::with_message(
            ErrorCode::ExtendPoint,
            message,
            Some(loc.clone()),
        ));
    }

    fn tag_in(tag: &str, candidates: &[&str]) -> bool {
        candidates
            .iter()
            .any(|candidate| tag.eq_ignore_ascii_case(candidate))
    }

    pub(super) fn note_implicitly_closed_stack_entries_from(&mut self, index: usize) {
        for entry in self.stack.iter().skip(index) {
            if !entry.implicit && is_html_tree_element(&entry.element) {
                self.implicitly_closed_tags.push(entry.element.tag);
            }
        }
    }

    pub(super) fn consume_implicitly_closed_tag(&mut self, tag: &str) -> bool {
        let Some(index) = self
            .implicitly_closed_tags
            .iter()
            .rposition(|closed| closed.eq_ignore_ascii_case(tag))
        else {
            return false;
        };
        self.implicitly_closed_tags.remove(index);
        true
    }
}

pub(super) fn is_html_tree_element(element: &ElementNode<'_>) -> bool {
    element.ns == Namespace::Html && element.tag_type == ElementType::Element
}

pub(super) fn note_html_tree_element_open(parser: &mut Parser<'_>, element: &ElementNode<'_>) {
    if !is_html_tree_element(element) {
        return;
    }

    match element.tag.len() {
        1 if element.tag.eq_ignore_ascii_case("p") => parser.open_p_count += 1,
        1 if element.tag.eq_ignore_ascii_case("a") => parser.open_a_count += 1,
        4 if element.tag.eq_ignore_ascii_case("form") => parser.open_form_count += 1,
        5 if element.tag.eq_ignore_ascii_case("table") => parser.open_table_count += 1,
        6 if element.tag.eq_ignore_ascii_case("button") => parser.open_button_count += 1,
        _ => {}
    }
}

pub(super) fn note_html_tree_element_close(parser: &mut Parser<'_>, element: &ElementNode<'_>) {
    if !is_html_tree_element(element) {
        return;
    }

    match element.tag.len() {
        1 if element.tag.eq_ignore_ascii_case("p") => {
            parser.open_p_count = parser.open_p_count.saturating_sub(1);
        }
        1 if element.tag.eq_ignore_ascii_case("a") => {
            parser.open_a_count = parser.open_a_count.saturating_sub(1);
        }
        4 if element.tag.eq_ignore_ascii_case("form") => {
            parser.open_form_count = parser.open_form_count.saturating_sub(1);
        }
        5 if element.tag.eq_ignore_ascii_case("table") => {
            parser.open_table_count = parser.open_table_count.saturating_sub(1);
        }
        6 if element.tag.eq_ignore_ascii_case("button") => {
            parser.open_button_count = parser.open_button_count.saturating_sub(1);
        }
        _ => {}
    }
}
