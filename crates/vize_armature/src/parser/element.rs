//! Element processing methods for the parser.
//!
//! Handles text, interpolation, open/close tags, element type determination,
//! comments, and error reporting.

use vize_carton::{directive::parse_vize_directive, Box};
use vize_relief::{
    ast::*,
    errors::{CompilerError, ErrorCode},
};

use super::{CurrentElement, Parser, ParserStackEntry};

impl<'a> Parser<'a> {
    /// Process text content
    pub(super) fn on_text_impl(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let content = self.get_source(start, end);
        let loc = self.create_loc(start, end);

        let text_node = TextNode::new(content, loc);
        let boxed = Box::new_in(text_node, self.allocator);
        self.add_child(TemplateChildNode::Text(boxed));
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
        let ns =
            (self.options.get_namespace)(tag, self.stack.last().map(|e| e.element.tag.as_str()));

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
            let loc = self.create_loc(tag_start - 1, end + 1); // Include < and >

            let mut element = ElementNode::new(self.allocator, current.tag.clone(), loc);
            element.ns = current.ns;
            element.is_self_closing = current.is_self_closing;
            element.props = current.props;

            // Determine element type
            element.tag_type = self.determine_element_type(&element);

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

            if current.is_self_closing || (self.options.is_void_tag)(element.tag.as_str()) {
                // Self-closing or void tag, add directly
                let boxed = Box::new_in(element, self.allocator);
                self.add_child(TemplateChildNode::Element(boxed));
            } else {
                // Push to stack
                self.stack.push(ParserStackEntry {
                    element,
                    in_pre: self.in_pre,
                    in_v_pre: self.in_v_pre,
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
        let tag = self.get_source(start, end);

        // Find matching open tag
        let mut found = false;
        for i in (0..self.stack.len()).rev() {
            if self.stack[i].element.tag.eq_ignore_ascii_case(tag) {
                found = true;

                // Pop all elements up to and including the match
                let mut elements: vize_carton::Vec<'a, ParserStackEntry<'a>> =
                    vize_carton::Vec::new_in(self.allocator);
                while self.stack.len() > i {
                    elements.push(self.stack.pop().unwrap());
                }

                // Report errors for unclosed elements (except the matching one)
                for entry in elements.iter().skip(1) {
                    let loc = entry.element.loc.clone();
                    self.errors
                        .push(CompilerError::new(ErrorCode::MissingEndTag, Some(loc)));
                }

                // Add all popped elements back as children
                for entry in elements.into_iter().rev() {
                    let in_pre = entry.in_pre;
                    let in_v_pre = entry.in_v_pre;

                    let boxed = Box::new_in(entry.element, self.allocator);
                    self.add_child(TemplateChildNode::Element(boxed));

                    self.in_pre = in_pre;
                    self.in_v_pre = in_v_pre;
                }

                break;
            }
        }

        if !found {
            let loc = self.create_loc(start - 2, end + 1); // Include </ and >
            self.errors
                .push(CompilerError::new(ErrorCode::InvalidEndTag, Some(loc)));
        }
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
        if let Some(is_custom) = self.options.is_custom_element {
            if is_custom(tag) {
                return false;
            }
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

    /// Process comment
    pub(super) fn on_comment_impl(&mut self, start: usize, end: usize) {
        let content = self.get_source(start, end);
        let loc = self.create_loc(start - 4, end + 3); // Include <!-- and -->

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

    /// Handle error
    pub(super) fn on_error_impl(&mut self, code: ErrorCode, index: usize) {
        let loc = self.create_loc(index, index + 1);
        self.errors.push(CompilerError::new(code, Some(loc)));
    }
}
