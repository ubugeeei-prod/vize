//! Vue template parser.
//!
//! This parser uses the tokenizer to convert Vue templates into an AST.
//! It is split into submodules for organization:
//! - `element` - Element, text, interpolation, comment, and error processing
//! - `attribute` - Attribute and directive processing
//! - `callbacks` - Tokenizer callback implementation
//! - `whitespace` - Whitespace condensing logic

mod attribute;
mod callbacks;
mod delimiters;
mod element;
mod entry;
#[cfg(test)]
mod experimental_tests;
mod whitespace;

pub use entry::*;

#[cfg(test)]
mod tests;

use vize_carton::{Bump, String, Vec};
use vize_relief::{
    ElementNode, Namespace, Position, PropNode, RootNode, SourceLocation, TemplateChildNode,
    errors::{CompilerError, ErrorCode},
    options::{ParserOptions, TemplateSyntaxMode, WhitespaceStrategy},
};

use element::{note_html_tree_element_close, note_html_tree_element_open};
use whitespace::condense_whitespace;

pub struct Parser<'a> {
    allocator: &'a Bump,
    source: &'a str,
    options: ParserOptions,
    /// Template syntax compatibility mode.
    template_syntax: TemplateSyntaxMode,
    /// Current node stack
    stack: Vec<'a, ParserStackEntry<'a>>,
    /// Tags of the elements the nesting limit refused to descend into, in source
    /// order. They are attached to the tree as leaves instead of being pushed
    /// onto `stack`, so without this their end tags would find nothing to close
    /// and be reported as `InvalidEndTag` even though the source is correct.
    flattened_tags: Vec<'a, String>,
    /// Root node
    root: Option<RootNode<'a>>,
    /// Current element being parsed
    current_element: Option<CurrentElement<'a>>,
    /// Current attribute being parsed
    current_attr: Option<CurrentAttribute<'a>>,
    /// Current directive being parsed
    current_dir: Option<CurrentDirective<'a>>,
    /// Errors collected during parsing
    errors: Vec<'a, CompilerError>,
    /// Newline positions for calculating line/column
    newlines: Vec<'a, usize>,
    /// Whether in pre block
    in_pre: bool,
    /// Whether in v-pre block
    in_v_pre: bool,
    open_table_count: usize,
    open_p_count: usize,
    open_a_count: usize,
    open_button_count: usize,
    open_form_count: usize,
    /// Whether the parser is in full-HTML-document mode (petite-vue / standalone
    /// HTML). When set, the tokenizer tolerates the leading doctype declaration
    /// so a real document's `<!DOCTYPE html>` is not reported as a parse error.
    /// SFC `<template>` parsing leaves this `false` and stays byte-identical.
    document: bool,
}

/// Stack entry for tracking parent elements
#[derive(Debug)]
pub(super) struct ParserStackEntry<'a> {
    pub(super) element: ElementNode<'a>,
    pub(super) in_pre: bool,
    pub(super) in_v_pre: bool,
    pub(super) insertion: StackInsertion,
    pub(super) implicit: bool,
    pub(super) fostered_before: Vec<'a, TemplateChildNode<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StackInsertion {
    Normal,
    Fostered,
}

/// Current element being parsed
pub(super) struct CurrentElement<'a> {
    pub(super) tag: String,
    pub(super) tag_start: usize,
    #[allow(dead_code)]
    pub(super) tag_end: usize,
    pub(super) ns: Namespace,
    pub(super) is_self_closing: bool,
    pub(super) props: Vec<'a, PropNode<'a>>,
}

/// Current attribute being parsed
pub(super) struct CurrentAttribute<'a> {
    pub(super) name: String,
    pub(super) name_start: usize,
    pub(super) name_end: usize,
    pub(super) value_start: Option<usize>,
    pub(super) value_end: Option<usize>,
    pub(super) value_content: Option<String>,
    pub(super) _marker: std::marker::PhantomData<&'a ()>,
}

/// Current directive being parsed
pub(super) struct CurrentDirective<'a> {
    pub(super) name: String,
    pub(super) raw_name: String,
    pub(super) name_start: usize,
    #[allow(dead_code)]
    pub(super) name_end: usize,
    pub(super) arg: Option<(String, usize, usize, bool)>, // (content, start, end, is_dynamic)
    pub(super) modifiers: Vec<'a, (String, usize, usize)>,
    pub(super) value_start: Option<usize>,
    pub(super) value_end: Option<usize>,
    pub(super) value_content: Option<String>,
    pub(super) _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(allocator: &'a Bump, source: &'a str) -> Self {
        Self::with_options(allocator, source, ParserOptions::default())
    }

    /// Create a new parser with options
    pub fn with_options(allocator: &'a Bump, source: &'a str, options: ParserOptions) -> Self {
        Self::with_options_and_template_syntax(
            allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
        )
    }

    /// Create a new parser with options and invalid HTML self-closing compatibility.
    #[deprecated(note = "use with_options_and_template_syntax instead")]
    pub fn with_options_and_invalid_html_self_closing(
        allocator: &'a Bump,
        source: &'a str,
        options: ParserOptions,
        allow_invalid_html_self_closing: bool,
    ) -> Self {
        Self::with_options_and_template_syntax(
            allocator,
            source,
            options,
            if allow_invalid_html_self_closing {
                TemplateSyntaxMode::Quirks
            } else {
                TemplateSyntaxMode::Standard
            },
        )
    }

    /// Create a new parser with options and template syntax compatibility.
    pub fn with_options_and_template_syntax(
        allocator: &'a Bump,
        source: &'a str,
        options: ParserOptions,
        template_syntax: TemplateSyntaxMode,
    ) -> Self {
        Self {
            allocator,
            source,
            options,
            template_syntax,
            stack: Vec::new_in(allocator),
            flattened_tags: Vec::new_in(allocator),
            root: None,
            current_element: None,
            current_attr: None,
            current_dir: None,
            errors: Vec::new_in(allocator),
            newlines: Vec::new_in(allocator),
            in_pre: false,
            in_v_pre: false,
            open_table_count: 0,
            open_p_count: 0,
            open_a_count: 0,
            open_button_count: 0,
            open_form_count: 0,
            document: false,
        }
    }

    /// Create a new parser in full-HTML-document mode.
    ///
    /// Document mode is additive: it parses an entire HTML document (doctype +
    /// `<html>/<head>/<body>`, with `<script>`/`<style>` kept as raw text) into
    /// the same template AST, so downstream analysis (lint/scope) can run over a
    /// petite-vue HTML page where directives (`v-scope`, `v-effect`, `@click`)
    /// live on ordinary elements. The only behavioral difference from
    /// [`Parser::with_options`] is doctype tolerance; SFC `<template>` parsing is
    /// unaffected.
    pub fn new_document(allocator: &'a Bump, source: &'a str) -> Self {
        Self::document_with_options(allocator, source, ParserOptions::default())
    }

    /// Create a new document-mode parser with options.
    pub fn document_with_options(
        allocator: &'a Bump,
        source: &'a str,
        options: ParserOptions,
    ) -> Self {
        let mut parser = Self::with_options_and_template_syntax(
            allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
        );
        parser.document = true;
        parser
    }

    /// Parse the source and return the AST
    pub fn parse(mut self) -> (RootNode<'a>, Vec<'a, CompilerError>) {
        // Initialize root node
        let root = RootNode::new(self.allocator, self.source);
        self.root = Some(root);

        if !self.tokenize_template() {
            return self.into_result();
        }

        // Handle any unclosed elements
        self.handle_unclosed_elements();

        // Condense whitespace if needed
        if let Some(ref mut root) = self.root
            && self.options.whitespace == WhitespaceStrategy::Condense
        {
            condense_whitespace(&mut root.children, self.options.is_pre_tag);
        }

        self.into_result()
    }

    /// Get source slice
    fn get_source(&self, start: usize, end: usize) -> &str {
        let (start, end) = self.normalize_span(start, end);
        &self.source[start..end]
    }

    fn normalize_span(&self, start: usize, end: usize) -> (usize, usize) {
        let mut start = self.clamp_to_char_boundary(start);
        let end = self.clamp_to_char_boundary(end);
        if start > end {
            start = end;
        }
        (start, end)
    }

    fn clamp_to_char_boundary(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.source.len());
        while offset > 0 && !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// Calculate position from byte offset
    fn get_pos(&self, offset: usize) -> Position {
        let line = match self.newlines.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i + 1,
        };

        let column = if line == 1 {
            offset + 1
        } else if line > 1 && line - 2 < self.newlines.len() {
            offset - self.newlines[line - 2]
        } else {
            offset + 1
        };

        Position::new(offset as u32, line as u32, column as u32)
    }

    /// Create a source location
    fn create_loc(&self, start: usize, end: usize) -> SourceLocation {
        let (start, end) = self.normalize_span(start, end);
        SourceLocation::new(self.get_pos(start), self.get_pos(end))
    }

    /// Add child to current context (stack top or root)
    fn add_child(&mut self, child: TemplateChildNode<'a>) {
        if let Some(entry) = self.stack.last_mut() {
            entry.element.children.push(child);
        } else if let Some(ref mut root) = self.root {
            root.children.push(child);
        }
    }

    fn add_fostered_child(&mut self, child: TemplateChildNode<'a>) {
        if let Some(table_index) = self.nearest_table_index() {
            self.stack[table_index].fostered_before.push(child);
        } else {
            self.add_child(child);
        }
    }

    pub(super) fn push_stack_entry(&mut self, entry: ParserStackEntry<'a>) {
        note_html_tree_element_open(self, &entry.element);
        self.stack.push(entry);
    }

    pub(super) fn pop_stack_entry(&mut self) -> Option<ParserStackEntry<'a>> {
        let entry = self.stack.pop()?;
        note_html_tree_element_close(self, &entry.element);
        Some(entry)
    }

    /// Handle unclosed elements at end of parsing
    fn handle_unclosed_elements(&mut self) {
        while let Some(entry) = self.pop_stack_entry() {
            if !entry.implicit && !Self::can_omit_end_tag(entry.element.tag.as_str()) {
                let loc = entry.element.loc.clone();
                self.errors
                    .push(CompilerError::new(ErrorCode::MissingEndTag, Some(loc)));
            }

            self.emit_stack_entry(entry);
        }
    }
}
