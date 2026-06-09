//! Element processing methods for the parser.
//!
//! Handles text, interpolation, open/close tags, element type determination,
//! comments, and error reporting.

use vize_carton::{Box, String, Vec, appends, directive::parse_vize_directive};
use vize_relief::{
    ast::*,
    errors::{CompilerError, ErrorCode},
    options::TemplateSyntaxMode,
};

use super::{CurrentElement, Parser, ParserStackEntry, StackInsertion};

/// Maximum element nesting depth retained by the parser.
///
/// Elements nested deeper than this are flattened with a recoverable error
/// instead of being pushed onto the open-element stack. This keeps the depth
/// of the produced AST bounded so the recursive passes that walk it later
/// (transform, codegen, semantic analysis) stay within a predictable amount of
/// stack space regardless of the input. The limit is far beyond any realistic
/// template while still cheap to enforce.
const MAX_ELEMENT_NESTING_DEPTH: usize = 256;

/// Message attached to the recoverable error raised when the nesting limit is hit.
const NESTING_TOO_DEEP_MESSAGE: &str = "Element nesting is too deep.";

impl<'a> Parser<'a> {
    /// Process text content
    pub(super) fn on_text_impl(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let source = self.get_source(start, end).to_owned();
        self.append_or_merge_text(&source, start, end);
    }

    /// Process text entity content
    pub(super) fn on_text_entity_impl(&mut self, ch: char, start: usize, end: usize) {
        let mut content = [0_u8; 4];
        self.append_or_merge_text(ch.encode_utf8(&mut content), start, end);
    }

    /// Append or merge text node
    fn append_or_merge_text(&mut self, content: &str, start: usize, end: usize) {
        if self.should_foster_text(content) {
            self.append_or_merge_fostered_text(content, start, end);
            return;
        }

        let merge_start_off = if let Some(entry) = self.stack.last() {
            match entry.element.children.last() {
                Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
                _ => None,
            }
        } else {
            match self.root.as_ref().and_then(|root| root.children.last()) {
                Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
                _ => None,
            }
        };

        if let Some(merge_start) = merge_start_off {
            let end_pos = self.get_pos(end);
            let source_span = self.get_source(merge_start, end).into();
            if let Some(entry) = self.stack.last_mut()
                && let Some(TemplateChildNode::Text(text_node)) = entry.element.children.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            } else if let Some(root) = self.root.as_mut()
                && let Some(TemplateChildNode::Text(text_node)) = root.children.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            }
        } else {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, self.allocator);
            self.add_child(TemplateChildNode::Text(boxed));
        }
    }

    fn append_or_merge_fostered_text(&mut self, content: &str, start: usize, end: usize) {
        let Some(table_index) = self.nearest_table_index() else {
            self.append_or_merge_text(content, start, end);
            return;
        };

        let merge_start_off = match self.stack[table_index].fostered_before.last() {
            Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
            _ => None,
        };

        if let Some(merge_start) = merge_start_off {
            let end_pos = self.get_pos(end);
            let source_span = self.get_source(merge_start, end).into();
            if let Some(TemplateChildNode::Text(text_node)) =
                self.stack[table_index].fostered_before.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            }
        } else {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, self.allocator);
            self.stack[table_index]
                .fostered_before
                .push(TemplateChildNode::Text(boxed));
        }
    }

    /// Process interpolation
    pub(super) fn on_interpolation_impl(&mut self, start: usize, end: usize) {
        let raw_content = self.get_source(start, end);
        let content = raw_content.trim();

        // Calculate trimmed positions for accurate source mapping
        let leading_ws = raw_content.len() - raw_content.trim_start().len();
        let trimmed_start = start + leading_ws;
        let trimmed_end = trimmed_start + content.len();

        let delim_len = self.options.delimiters.0.len();
        let full_start = start - delim_len;
        let full_end = end + self.options.delimiters.1.len();
        let loc = self.create_loc(full_start, full_end);
        let inner_loc = self.create_loc(trimmed_start, trimmed_end);

        // Create expression node
        let expr = SimpleExpressionNode::new(content, false, inner_loc);
        let expr_boxed = Box::new_in(expr, self.allocator);

        let interp = InterpolationNode {
            content: ExpressionNode::Simple(expr_boxed),
            loc,
        };
        let boxed = Box::new_in(interp, self.allocator);
        self.add_child(TemplateChildNode::Interpolation(boxed));
    }

    /// Process open tag name
    pub(super) fn on_open_tag_name_impl(&mut self, start: usize, end: usize) {
        let tag = self.get_source(start, end);
        let parent = self.stack.last().map(|e| e.element.tag.as_str());
        let ns = if self.should_force_html_namespace(tag) {
            Namespace::Html
        } else {
            let resolved = (self.options.get_namespace)(tag, parent);
            // Resolve the foreign (SVG/MathML) namespace even when the
            // configured `get_namespace` callback is namespace-unaware (the
            // default callback always returns HTML). Without this, the
            // `vize_canon` virtual-TS path — which parses with
            // `ParserOptions::default()` — leaves `<svg>`/`<math>` and their
            // self-closing children such as `<path d="…" />` in the HTML
            // namespace and then flags them as invalid self-closing non-void
            // HTML elements.
            if resolved == Namespace::Html {
                self.foreign_namespace_for(tag).unwrap_or(Namespace::Html)
            } else {
                resolved
            }
        };

        self.current_element = Some(CurrentElement {
            tag: tag.into(),
            tag_start: start,
            tag_end: end,
            ns,
            is_self_closing: false,
            props: vize_carton::Vec::new_in(self.allocator),
        });
    }

    /// Process open tag end
    pub(super) fn on_open_tag_end_impl(&mut self, end: usize) {
        if let Some(current) = self.current_element.take() {
            let tag_start = current.tag_start;
            let loc = self.create_loc(tag_start.saturating_sub(1), end + 1); // Include < and >

            let mut element = ElementNode::new(self.allocator, current.tag.clone(), loc);
            element.ns = current.ns;
            element.is_self_closing = current.is_self_closing;
            element.props = current.props;

            // Determine element type
            element.tag_type = self.determine_element_type(&element);

            if self.should_ignore_start_tag(&element) {
                self.report_tree_construction_recovery(
                    &element.loc,
                    "HTML tree construction ignored this start tag because an equivalent element is already open.",
                );
                return;
            }

            self.handle_in_body_start_tag(element.tag.as_str(), tag_start);
            self.handle_in_table_start_tag(element.tag.as_str(), tag_start);

            // Check for pre tags
            let is_pre = (self.options.is_pre_tag)(element.tag.as_str());
            let has_v_pre = element
                .props
                .iter()
                .any(|p| matches!(p, PropNode::Directive(d) if d.name == "pre"));

            // When v-pre is on this element, convert all directives (except v-pre itself)
            // back to raw attribute nodes, since v-pre means "skip compilation"
            if has_v_pre {
                let allocator = self.allocator;
                let mut i = 0;
                while i < element.props.len() {
                    if let PropNode::Directive(dir) = &element.props[i] {
                        if dir.name == "pre" {
                            // Remove v-pre directive itself
                            element.props.remove(i);
                            continue;
                        }
                        // Convert directive back to attribute using its raw_name + arg
                        // to reconstruct the original attribute name (e.g., ":id", "@click")
                        let attr_name = {
                            let prefix = dir.raw_name.as_deref().unwrap_or(&dir.name);
                            let arg_str = dir.arg.as_ref().map(|a| match a {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            });
                            if let Some(arg) = arg_str {
                                let mut name =
                                    vize_carton::String::with_capacity(prefix.len() + arg.len());
                                name.push_str(prefix);
                                name.push_str(arg);
                                name
                            } else {
                                vize_carton::String::from(prefix)
                            }
                        };
                        let attr_value = dir.exp.as_ref().map(|e| {
                            let content = match e {
                                ExpressionNode::Simple(s) => s.loc.source.clone(),
                                ExpressionNode::Compound(c) => c.loc.source.clone(),
                            };
                            TextNode {
                                content,
                                loc: dir.loc.clone(),
                            }
                        });
                        let attr = PropNode::Attribute(Box::new_in(
                            AttributeNode {
                                name: attr_name,
                                name_loc: dir.loc.clone(),
                                value: attr_value,
                                loc: dir.loc.clone(),
                            },
                            allocator,
                        ));
                        element.props[i] = attr;
                    }
                    i += 1;
                }
            }

            let html_non_void_self_closing =
                current.is_self_closing && self.is_invalid_html_self_closing(&element);

            if html_non_void_self_closing {
                match self.template_syntax {
                    TemplateSyntaxMode::Standard => {
                        self.report_tree_construction_recovery(
                            &element.loc,
                            "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.",
                        );
                        element.is_self_closing = false;
                    }
                    TemplateSyntaxMode::Strict => {
                        self.errors.push(CompilerError::with_message(
                            ErrorCode::UnexpectedSolidusInTag,
                            "Invalid self-closing syntax on non-void HTML element.",
                            Some(element.loc.clone()),
                        ));
                        element.is_self_closing = false;
                    }
                    TemplateSyntaxMode::Quirks => {
                        element.is_self_closing = true;
                    }
                    _ => {
                        self.report_tree_construction_recovery(
                            &element.loc,
                            "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.",
                        );
                        element.is_self_closing = false;
                    }
                }
            }

            if current.is_self_closing || (self.options.is_void_tag)(element.tag.as_str()) {
                let should_foster_direct = self.should_foster_start_tag(
                    element.tag.as_str(),
                    element.ns == Namespace::Html && element.tag_type == ElementType::Element,
                );
                // Self-closing or void tag, add directly
                let boxed = Box::new_in(element, self.allocator);
                let child = TemplateChildNode::Element(boxed);
                if should_foster_direct {
                    self.add_fostered_child(child);
                } else {
                    self.add_child(child);
                }
            } else if self.stack.len() >= MAX_ELEMENT_NESTING_DEPTH {
                // Nesting limit reached: keep the element but do not descend any
                // further, so the resulting tree depth stays bounded. The
                // element is attached at the current level as a leaf and a
                // recoverable error is recorded.
                self.errors.push(CompilerError::with_message(
                    ErrorCode::ExtendPoint,
                    NESTING_TOO_DEEP_MESSAGE,
                    Some(element.loc.clone()),
                ));
                let boxed = Box::new_in(element, self.allocator);
                self.add_child(TemplateChildNode::Element(boxed));
            } else {
                let insertion = if self.should_foster_start_tag(element.tag.as_str(), true) {
                    self.report_tree_construction_recovery(
                        &element.loc,
                        "Foster parenting moved this element before the nearest open table.",
                    );
                    StackInsertion::Fostered
                } else {
                    StackInsertion::Normal
                };
                // Push to stack
                self.push_stack_entry(ParserStackEntry {
                    element,
                    in_pre: self.in_pre,
                    in_v_pre: self.in_v_pre,
                    insertion,
                    implicit: false,
                    fostered_before: Vec::new_in(self.allocator),
                });
                self.in_pre = is_pre || self.in_pre;
                self.in_v_pre = has_v_pre || self.in_v_pre;
            }
        }
    }

    /// Process self-closing tag
    pub(super) fn on_self_closing_tag_impl(&mut self, _end: usize) {
        if let Some(ref mut current) = self.current_element {
            current.is_self_closing = true;
        }
    }

    /// Process close tag
    pub(super) fn on_close_tag_impl(&mut self, start: usize, end: usize) {
        let tag = self.get_source(start, end).to_owned();

        if self.handle_formatting_end_tag(tag.as_str(), start, end) {
            return;
        }

        // Find matching open tag
        if let Some(i) = self.find_open_element_index(tag.as_str()) {
            self.close_stack_element_at(i, true);
            return;
        }

        let loc = self.create_loc(start.saturating_sub(2), end + 1); // Include </ and >
        self.errors
            .push(CompilerError::new(ErrorCode::InvalidEndTag, Some(loc)));
    }

    pub(super) fn emit_stack_entry(&mut self, entry: ParserStackEntry<'a>) {
        let insertion = entry.insertion;
        let mut nodes = entry.fostered_before;
        let boxed = Box::new_in(entry.element, self.allocator);
        nodes.push(TemplateChildNode::Element(boxed));

        for node in nodes {
            if insertion == StackInsertion::Fostered {
                self.add_fostered_child(node);
            } else {
                self.add_child(node);
            }
        }
    }

    fn close_stack_element_at(&mut self, index: usize, report_unclosed: bool) {
        let mut entries = Vec::new_in(self.allocator);
        while self.stack.len() > index {
            if let Some(entry) = self.pop_stack_entry() {
                entries.push(entry);
            }
        }

        let Some(target) = entries.last() else {
            return;
        };
        let restored_in_pre = target.in_pre;
        let restored_in_v_pre = target.in_v_pre;

        if report_unclosed {
            for entry in entries.iter().take(entries.len().saturating_sub(1)) {
                if !entry.implicit && !Self::can_omit_end_tag(entry.element.tag.as_str()) {
                    let loc = entry.element.loc.clone();
                    self.errors
                        .push(CompilerError::new(ErrorCode::MissingEndTag, Some(loc)));
                }
            }
        }

        let mut child: Option<ParserStackEntry<'a>> = None;
        for mut entry in entries {
            if let Some(child_entry) = child.take() {
                self.push_entry_as_child(&mut entry.element.children, child_entry);
            }
            child = Some(entry);
        }

        if let Some(entry) = child {
            self.emit_stack_entry(entry);
        }

        self.in_pre = restored_in_pre;
        self.in_v_pre = restored_in_v_pre;
    }

    fn push_entry_as_child(
        &mut self,
        children: &mut Vec<'a, TemplateChildNode<'a>>,
        entry: ParserStackEntry<'a>,
    ) {
        for node in entry.fostered_before {
            children.push(node);
        }
        let boxed = Box::new_in(entry.element, self.allocator);
        children.push(TemplateChildNode::Element(boxed));
    }

    fn find_open_element_index(&self, tag: &str) -> Option<usize> {
        (0..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].element.tag.eq_ignore_ascii_case(tag))
    }

    pub(super) fn nearest_table_index(&self) -> Option<usize> {
        if self.open_table_count == 0 {
            return None;
        }

        (0..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].element.tag.eq_ignore_ascii_case("table"))
    }

    fn should_foster_text(&self, content: &str) -> bool {
        self.open_table_count > 0
            && content
                .chars()
                .any(|c| !matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000C}'))
            && self.is_in_table_insertion_context()
    }

    fn should_foster_start_tag(&self, tag: &str, is_html_element: bool) -> bool {
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

    fn handle_in_table_start_tag(&mut self, tag: &str, offset: usize) {
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

    fn handle_in_body_start_tag(&mut self, tag: &str, offset: usize) {
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

    fn find_last_open_element_index(&self, tags: &[&str]) -> Option<usize> {
        (0..self.stack.len())
            .rev()
            .find(|&i| Self::tag_in(self.stack[i].element.tag.as_str(), tags))
    }

    fn find_open_li_element_in_list_item_scope(&self) -> Option<usize> {
        for i in (0..self.stack.len()).rev() {
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
            let tag = self.stack[i].element.tag.as_str();
            if tag.eq_ignore_ascii_case("p") {
                return Some(i);
            }
            // Components and structural `<template>` blocks (the latter is also
            // a button-scope boundary in the HTML spec) confine `<p>`
            // auto-closing, so stop the search rather than reaching across them.
            if self.stack[i].element.tag_type != ElementType::Element
                || Self::is_button_scope_boundary(tag)
            {
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

    fn should_ignore_start_tag(&self, element: &ElementNode<'a>) -> bool {
        element.tag.eq_ignore_ascii_case("form") && self.open_form_count > 0
    }

    fn is_invalid_html_self_closing(&self, element: &ElementNode<'a>) -> bool {
        element.ns == Namespace::Html
            && element.tag_type == ElementType::Element
            && (!self.options.custom_renderer || vize_carton::is_html_tag(element.tag.as_str()))
            && !(self.options.is_void_tag)(element.tag.as_str())
    }

    fn handle_formatting_end_tag(&mut self, tag: &str, start: usize, end: usize) -> bool {
        if !Self::is_formatting_tag(tag) {
            return false;
        }
        if self
            .stack
            .last()
            .is_some_and(|entry| entry.element.tag.eq_ignore_ascii_case(tag))
        {
            return false;
        }
        let Some(index) = self.find_open_element_index(tag) else {
            return false;
        };
        if index + 1 == self.stack.len() {
            return false;
        }

        let loc = self.create_loc(start.saturating_sub(2), end + 1);
        self.report_tree_construction_recovery(
            &loc,
            "Misnested formatting end tag was repaired using the adoption agency recovery path.",
        );

        let mut reopened = Vec::new_in(self.allocator);
        while self.stack.len() > index + 1 {
            if let Some(entry) = self.pop_stack_entry() {
                if Self::is_formatting_tag(entry.element.tag.as_str()) {
                    reopened.push(Self::formatting_shell(
                        self.allocator,
                        &entry.element,
                        self.in_pre,
                        self.in_v_pre,
                    ));
                }
                let mut parent = self
                    .pop_stack_entry()
                    .expect("formatting match is still open");
                self.push_entry_as_child(&mut parent.element.children, entry);
                self.push_stack_entry(parent);
            }
        }

        let match_index = self.stack.len() - 1;
        self.close_stack_element_at(match_index, false);

        for mut entry in reopened.into_iter().rev() {
            entry.in_pre = self.in_pre;
            entry.in_v_pre = self.in_v_pre;
            self.push_stack_entry(entry);
        }

        true
    }

    fn formatting_shell(
        allocator: &'a vize_carton::Bump,
        element: &ElementNode<'a>,
        in_pre: bool,
        in_v_pre: bool,
    ) -> ParserStackEntry<'a> {
        let mut reopened = ElementNode::new(allocator, element.tag.clone(), element.loc.clone());
        reopened.ns = element.ns;
        reopened.tag_type = element.tag_type;
        ParserStackEntry {
            element: reopened,
            in_pre,
            in_v_pre,
            insertion: StackInsertion::Normal,
            implicit: true,
            fostered_before: Vec::new_in(allocator),
        }
    }

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

    fn is_formatting_tag(tag: &str) -> bool {
        Self::tag_in(
            tag,
            &[
                "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                "strong", "tt", "u",
            ],
        )
    }

    pub(super) fn can_omit_end_tag(tag: &str) -> bool {
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

    /// Determine element type (element, component, slot, template)
    pub(super) fn determine_element_type(&self, element: &ElementNode<'a>) -> ElementType {
        let tag = element.tag.as_str();

        // Check for slot
        if tag == "slot" {
            return ElementType::Slot;
        }

        // Check for template
        if tag == "template" {
            // Template with v-if, v-for, or v-slot is a template element
            let has_structural_directive = element.props.iter().any(|p| {
                matches!(p, PropNode::Directive(d) if matches!(d.name.as_str(), "if" | "else-if" | "else" | "for" | "slot"))
            });
            if has_structural_directive {
                return ElementType::Template;
            }
        }

        // Check if it's a component
        if self.is_component(tag) {
            return ElementType::Component;
        }

        ElementType::Element
    }

    /// Check if tag is a component
    pub(super) fn is_component(&self, tag: &str) -> bool {
        // Core built-in components
        if matches!(
            tag,
            "Teleport"
                | "Suspense"
                | "KeepAlive"
                | "BaseTransition"
                | "Transition"
                | "TransitionGroup"
        ) {
            return true;
        }

        // Custom element check
        if let Some(is_custom) = self.options.is_custom_element
            && is_custom(tag)
        {
            return false;
        }

        if self.options.custom_renderer {
            return tag.chars().next().is_some_and(|c| c.is_uppercase()) || tag.contains('-');
        }

        // Native tag check
        if let Some(is_native) = self.options.is_native_tag {
            if !is_native(tag) {
                return true;
            }
        } else {
            // Default: check if starts with uppercase
            if tag.chars().next().is_some_and(|c| c.is_uppercase()) {
                return true;
            }
        }

        false
    }

    /// Resolve the foreign (SVG/MathML) namespace for a start tag whose
    /// configured `get_namespace` callback returned HTML. An `<svg>`/`<math>`
    /// root (or any SVG/MathML tag) seeds the namespace; otherwise descendants
    /// inherit the nearest open ancestor's foreign namespace unless that
    /// ancestor is an HTML integration point (`<foreignObject>`/`<desc>`/
    /// `<title>` for SVG, `<annotation-xml>` and the MathML text containers for
    /// MathML), which switch their subtree back to HTML. Mirrors the boundary
    /// handling in the DOM compiler's `get_namespace` so namespace-unaware
    /// callbacks still classify foreign elements correctly.
    fn foreign_namespace_for(&self, tag: &str) -> Option<Namespace> {
        if vize_carton::is_svg_tag(tag) {
            return Some(Namespace::Svg);
        }
        if vize_carton::is_math_ml_tag(tag) {
            return Some(Namespace::MathMl);
        }

        let parent = self.stack.last()?;
        let parent_tag = parent.element.tag.as_str();
        match parent.element.ns {
            Namespace::Svg => {
                let svg_to_html = matches!(parent_tag, "foreignObject" | "desc" | "title");
                (!svg_to_html).then_some(Namespace::Svg)
            }
            Namespace::MathMl => {
                let mathml_to_html = matches!(
                    parent_tag,
                    "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
                );
                (!mathml_to_html).then_some(Namespace::MathMl)
            }
            Namespace::Html => None,
        }
    }

    fn should_force_html_namespace(&self, tag: &str) -> bool {
        if !self.options.custom_renderer {
            return false;
        }

        if matches!(tag, "svg" | "math") {
            return false;
        }

        if self
            .stack
            .last()
            .is_some_and(|entry| matches!(entry.element.ns, Namespace::Svg | Namespace::MathMl))
        {
            return false;
        }

        tag.chars().next().is_some_and(|c| c.is_lowercase())
            && !tag.contains('-')
            && !vize_carton::is_html_tag(tag)
    }

    /// Process comment
    pub(super) fn on_comment_impl(&mut self, start: usize, end: usize) {
        let content = self.get_source(start, end);
        let loc_start = start.saturating_sub(4);
        let loc_end = self.comment_loc_end(start, end);
        let loc = self.create_loc(loc_start, loc_end); // Include <!-- and --> when present.

        // Check for @vize: directive
        let directive = parse_vize_directive(content, loc.start.line, loc.start.offset);

        // Always preserve directive comments (even when options.comments = false)
        // so they can be explicitly handled by codegen and linter
        if directive.is_none() && !self.options.comments {
            return;
        }

        let mut comment = CommentNode::new(content, loc);
        comment.directive = directive.map(|d| d.kind);
        let boxed = Box::new_in(comment, self.allocator);
        self.add_child(TemplateChildNode::Comment(boxed));
    }

    fn comment_loc_end(&self, start: usize, end: usize) -> usize {
        let end = self.clamp_to_char_boundary(end);
        let rest = &self.source[end..];
        if rest.starts_with("-->") {
            end + 3
        } else if start == end && rest.starts_with("->") {
            end + 2
        } else if start == end && rest.starts_with('>') {
            end + 1
        } else {
            end.saturating_add(3).min(self.source.len())
        }
    }

    /// Process CDATA
    pub(super) fn on_cdata_impl(&mut self, start: usize, end: usize) {
        let is_html_ns = self
            .stack
            .last()
            .map(|e| e.element.ns)
            .unwrap_or(Namespace::Html)
            == Namespace::Html;
        if is_html_ns {
            self.on_error_impl(ErrorCode::CdataInHtmlContent, start.saturating_sub(9));
        } else {
            self.on_text_impl(start, end);
        }
    }

    /// Handle error
    pub(super) fn on_error_impl(&mut self, code: ErrorCode, index: usize) {
        let len = self.source.len();
        let start = index.min(len);
        let end = (index + 1).min(len);
        let loc = self.create_loc(start, end);
        let error = if let Some(message) = self.recovery_error_message(code) {
            CompilerError::with_message(code, message, Some(loc))
        } else {
            CompilerError::new(code, Some(loc))
        };
        self.errors.push(error);
    }

    fn recovery_error_message(&self, code: ErrorCode) -> Option<String> {
        match code {
            ErrorCode::EofBeforeTagName => Some(
                "Unexpected end of input after `<`; treating it as text so parsing can continue."
                    .into(),
            ),
            ErrorCode::EofInTag => Some(
                "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue."
                    .into(),
            ),
            ErrorCode::EofInComment => Some(
                "Comment is missing its closing `-->`; preserving the unfinished comment so parsing can finish."
                    .into(),
            ),
            ErrorCode::InvalidFirstCharacterOfTagName => Some(
                "Tag name starts with an invalid character; treating the malformed tag as text.".into(),
            ),
            ErrorCode::MissingAttributeValue => {
                let name = self
                    .current_attr
                    .as_ref()
                    .map(|attr| attr.name.as_str())
                    .or_else(|| self.current_dir.as_ref().map(|dir| dir.raw_name.as_str()))
                    .unwrap_or("attribute");
                let mut message = String::with_capacity(name.len() + 70);
                appends!(
                    message,
                    "Attribute `",
                    name,
                    "` is missing a value after `=`; continuing without the value."
                );
                Some(message)
            }
            ErrorCode::MissingDynamicDirectiveArgumentEnd => Some(
                "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary."
                    .into(),
            ),
            ErrorCode::MissingInterpolationEnd => {
                let delimiter = self.options.delimiters.1.as_str();
                let mut message = String::with_capacity(delimiter.len() + 97);
                appends!(
                    message,
                    "Interpolation is missing its closing delimiter `",
                    delimiter,
                    "`; treating the unfinished interpolation as text."
                );
                Some(message)
            }
            ErrorCode::UnexpectedCharacterInAttributeName => Some(
                "Attribute name contains an invalid character; inferred the nearest attribute boundary and continued."
                    .into(),
            ),
            ErrorCode::UnexpectedCharacterInUnquotedAttributeValue => Some(
                "Unquoted attribute value contains a character that should be quoted; keeping it in the value and continuing."
                    .into(),
            ),
            ErrorCode::UnexpectedEqualsSignBeforeAttributeName => Some(
                "Unexpected `=` before an attribute name; skipping it and continuing with the next attribute."
                    .into(),
            ),
            ErrorCode::MissingWhitespaceBetweenAttributes => Some(
                "Missing whitespace between attributes; inferred a new attribute boundary.".into(),
            ),
            ErrorCode::IncorrectlyClosedComment => Some(
                "Comment was closed as `--!>`; treating it as `-->` so parsing can continue."
                    .into(),
            ),
            ErrorCode::IncorrectlyOpenedComment => Some(
                "Declaration or comment syntax is malformed; skipping it until the next `>`.".into(),
            ),
            _ => None,
        }
    }
}
