//! Open/close tag processing and the open-element stack lifecycle.

use vize_carton::{Box, Vec};
use vize_relief::{
    AttributeNode, ElementNode, ElementType, ExpressionNode, Namespace, PropNode,
    TemplateChildNode, TextNode,
    errors::{CompilerError, ErrorCode},
    options::TemplateSyntaxMode,
};

use super::super::{CurrentElement, Parser, ParserStackEntry, StackInsertion};

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
    /// Process open tag name
    pub(in crate::parser) fn on_open_tag_name_impl(&mut self, start: usize, end: usize) {
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
    pub(in crate::parser) fn on_open_tag_end_impl(&mut self, end: usize) {
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
    pub(in crate::parser) fn on_self_closing_tag_impl(&mut self, _end: usize) {
        if let Some(ref mut current) = self.current_element {
            current.is_self_closing = true;
        }
    }

    /// Process close tag
    pub(in crate::parser) fn on_close_tag_impl(&mut self, start: usize, end: usize) {
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

    pub(in crate::parser) fn emit_stack_entry(&mut self, entry: ParserStackEntry<'a>) {
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

    pub(super) fn close_stack_element_at(&mut self, index: usize, report_unclosed: bool) {
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

    pub(super) fn push_entry_as_child(
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

    pub(super) fn find_open_element_index(&self, tag: &str) -> Option<usize> {
        (0..self.stack.len())
            .rev()
            .find(|&i| self.stack[i].element.tag.eq_ignore_ascii_case(tag))
    }
}
