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
mod element;
mod whitespace;

#[cfg(test)]
mod tests;

use vize_carton::{Bump, String, Vec};
use vize_relief::{
    ast::*,
    errors::{CompilerError, ErrorCode},
    options::{ParserOptions, WhitespaceStrategy},
};

use crate::tokenizer::Tokenizer;

use callbacks::ParserCallbacks;
use whitespace::condense_whitespace;

/// Parser context for building AST
pub struct Parser<'a> {
    /// Arena allocator
    allocator: &'a Bump,
    /// Source code
    source: &'a str,
    /// Parser options
    options: ParserOptions,
    /// Current node stack
    stack: Vec<'a, ParserStackEntry<'a>>,
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
        Self {
            allocator,
            source,
            options,
            stack: Vec::new_in(allocator),
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
        }
    }

    /// Parse the source and return the AST
    pub fn parse(mut self) -> (RootNode<'a>, Vec<'a, CompilerError>) {
        // Initialize root node
        let root = RootNode::new(self.allocator, self.source);
        self.root = Some(root);

        // Copy delimiters to avoid borrow issue
        let delimiter_open: Vec<'a, u8> =
            Vec::from_iter_in(self.options.delimiters.0.bytes(), self.allocator);
        let delimiter_close: Vec<'a, u8> =
            Vec::from_iter_in(self.options.delimiters.1.bytes(), self.allocator);

        // We need to use a struct that implements Callbacks
        // Create a wrapper that can capture the parser
        let mut tokenizer = Tokenizer::with_delimiters(
            self.source,
            ParserCallbacks { parser: &mut self },
            &delimiter_open,
            &delimiter_close,
        );
        tokenizer.tokenize();

        // Handle any unclosed elements
        self.handle_unclosed_elements();

        // Condense whitespace if needed
        if let Some(ref mut root) = self.root
            && self.options.whitespace == WhitespaceStrategy::Condense
        {
            condense_whitespace(&mut root.children, self.options.is_pre_tag);
        }

        let root = match self.root.take() {
            Some(root) => root,
            None => RootNode::new(self.allocator, self.source),
        };
        (root, self.errors)
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
        SourceLocation::new(
            self.get_pos(start),
            self.get_pos(end),
            self.get_source(start, end),
        )
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
        self.note_stack_tag_open(entry.element.tag.as_str());
        self.stack.push(entry);
    }

    pub(super) fn pop_stack_entry(&mut self) -> Option<ParserStackEntry<'a>> {
        let entry = self.stack.pop()?;
        self.note_stack_tag_close(entry.element.tag.as_str());
        Some(entry)
    }

    fn note_stack_tag_open(&mut self, tag: &str) {
        match tag.len() {
            1 if tag.eq_ignore_ascii_case("p") => self.open_p_count += 1,
            1 if tag.eq_ignore_ascii_case("a") => self.open_a_count += 1,
            4 if tag.eq_ignore_ascii_case("form") => self.open_form_count += 1,
            5 if tag.eq_ignore_ascii_case("table") => self.open_table_count += 1,
            6 if tag.eq_ignore_ascii_case("button") => self.open_button_count += 1,
            _ => {}
        }
    }

    fn note_stack_tag_close(&mut self, tag: &str) {
        match tag.len() {
            1 if tag.eq_ignore_ascii_case("p") => {
                self.open_p_count = self.open_p_count.saturating_sub(1);
            }
            1 if tag.eq_ignore_ascii_case("a") => {
                self.open_a_count = self.open_a_count.saturating_sub(1);
            }
            4 if tag.eq_ignore_ascii_case("form") => {
                self.open_form_count = self.open_form_count.saturating_sub(1);
            }
            5 if tag.eq_ignore_ascii_case("table") => {
                self.open_table_count = self.open_table_count.saturating_sub(1);
            }
            6 if tag.eq_ignore_ascii_case("button") => {
                self.open_button_count = self.open_button_count.saturating_sub(1);
            }
            _ => {}
        }
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

/// Parse a Vue template
pub fn parse<'a>(allocator: &'a Bump, source: &'a str) -> (RootNode<'a>, Vec<'a, CompilerError>) {
    Parser::new(allocator, source).parse()
}

/// Parse a Vue template with options
pub fn parse_with_options<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: ParserOptions,
) -> (RootNode<'a>, Vec<'a, CompilerError>) {
    Parser::with_options(allocator, source, options).parse()
}
