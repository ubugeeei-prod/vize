//! "In body" start-tag handling: implicit end-tag closing and HTML scopes
//! (`<a>`/`<button>`/`<li>`/`<dt>`/`<dd>`/`<option>`/`<optgroup>`/`<p>`).

use vize_relief::ElementNode;

use super::super::Parser;
use super::is_html_tree_element;

impl<'a> Parser<'a> {
    pub(super) fn handle_in_body_start_tag(&mut self, tag: &str, offset: usize) {
        if tag.eq_ignore_ascii_case("html") || tag.eq_ignore_ascii_case("body") {
            return;
        }

        if self.open_a_count == 0
            && self.open_button_count == 0
            && self.open_p_count == 0
            && !Self::starts_optional_end_tag_scope(tag)
        {
            return;
        }

        if tag.eq_ignore_ascii_case("a") {
            if self.open_a_count > 0
                && let Some(index) = self.find_open_element_index("a")
            {
                self.report_tree_construction_recovery(
                    &self.stack[index].element.loc.clone(),
                    "Nested anchor start tag closed the previous anchor before inserting the new one.",
                );
                self.close_stack_element_at(index, false);
            }
            return;
        }

        if tag.eq_ignore_ascii_case("button") {
            if self.open_button_count > 0
                && let Some(index) = self.find_open_element_index("button")
            {
                self.report_tree_construction_recovery(
                    &self.stack[index].element.loc.clone(),
                    "Nested button start tag closed the previous button before inserting the new one.",
                );
                self.close_stack_element_at(index, false);
            }
            return;
        }

        if tag.eq_ignore_ascii_case("li") {
            if let Some(index) = self.find_open_li_element_in_list_item_scope() {
                self.close_stack_element_at(index, false);
            }
        } else if Self::tag_in(tag, &["dt", "dd"]) {
            if let Some(index) = self.find_last_open_element_index(&["dt", "dd"]) {
                self.close_stack_element_at(index, false);
            }
        } else if tag.eq_ignore_ascii_case("option") {
            if let Some(index) = self.find_open_element_index("option") {
                self.close_stack_element_at(index, false);
            }
        } else if tag.eq_ignore_ascii_case("optgroup")
            && let Some(index) = self.find_last_open_element_index(&["option", "optgroup"])
        {
            self.close_stack_element_at(index, false);
        }

        if self.open_p_count > 0
            && (tag.eq_ignore_ascii_case("p") || Self::closes_open_p_before_start(tag))
            && let Some(index) = self.find_open_p_element_in_button_scope()
        {
            // Only auto-close a `<p>` that is in button scope of the current
            // insertion point. A `<template>` (or other scope-terminating
            // element) between the open `<p>` and the new start tag is a scope
            // boundary, so the `<p>` must NOT be auto-closed across it —
            // otherwise valid markup like `<p><template>…<p>…</p></template></p>`
            // wrongly reports an `InvalidEndTag` for the outer `</p>`.
            self.close_stack_element_at(index, false);
        }

        let _ = offset;
    }

    pub(super) fn find_last_open_element_index(&self, tags: &[&str]) -> Option<usize> {
        (0..self.stack.len()).rev().find(|&i| {
            is_html_tree_element(&self.stack[i].element)
                && Self::tag_in(self.stack[i].element.tag.as_str(), tags)
        })
    }

    fn find_open_li_element_in_list_item_scope(&self) -> Option<usize> {
        for i in (0..self.stack.len()).rev() {
            if !is_html_tree_element(&self.stack[i].element) {
                return None;
            }

            let tag = self.stack[i].element.tag.as_str();
            if tag.eq_ignore_ascii_case("li") {
                return Some(i);
            }
            if Self::is_list_item_scope_boundary(tag) {
                return None;
            }
        }

        None
    }

    fn find_open_p_element_in_button_scope(&self) -> Option<usize> {
        for i in (0..self.stack.len()).rev() {
            if !is_html_tree_element(&self.stack[i].element) {
                return None;
            }

            let tag = self.stack[i].element.tag.as_str();
            if tag.eq_ignore_ascii_case("p") {
                return Some(i);
            }
            // Components and structural `<template>` blocks (the latter is also
            // a button-scope boundary in the HTML spec) confine `<p>`
            // auto-closing, so stop the search rather than reaching across them.
            if Self::is_button_scope_boundary(tag) {
                return None;
            }
        }

        None
    }

    /// HTML "button scope" terminating elements (the default scope set plus
    /// `<button>`). A `<p>` start tag (or a `<p>`-closing block start tag) only
    /// auto-closes an open `<p>` that sits within this scope.
    fn is_button_scope_boundary(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "applet", "button", "caption", "html", "table", "td", "th", "marquee", "object",
                "template",
            ],
        )
    }

    fn is_list_item_scope_boundary(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
                "ol", "ul",
            ],
        )
    }

    pub(super) fn should_ignore_start_tag(&self, element: &ElementNode<'a>) -> bool {
        is_html_tree_element(element)
            && element.tag.eq_ignore_ascii_case("form")
            && self.open_form_count > 0
    }

    pub(in crate::parser) fn can_omit_end_tag(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "li", "dt", "dd", "p", "rt", "rp", "optgroup", "option", "thead", "tbody", "tfoot",
                "tr", "td", "th",
            ],
        )
    }

    fn closes_open_p_before_start(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "address",
                "article",
                "aside",
                "blockquote",
                "div",
                "dl",
                "fieldset",
                "footer",
                "form",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "header",
                "hr",
                "main",
                "nav",
                "ol",
                "p",
                "pre",
                "section",
                "table",
                "ul",
            ],
        )
    }

    fn starts_optional_end_tag_scope(tag: &str) -> bool {
        Self::tag_in(tag, &["li", "dt", "dd", "option", "optgroup"])
    }
}
